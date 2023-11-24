use anyhow::{anyhow, Context, Result};
use log::{error, info};
use reqwest;
use serde;
use serde_json::json;
use std::env;
use std::error::Error;
use std::fs;
use url::Url;
use xdg::BaseDirectories;

const BASE_URL: &str = "https://api.peerberry.com";

struct State {
    available_money: f64,
    loans: Vec<Loan>,
}

#[derive(serde::Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct LoginResponse {
    tfa_is_active: bool,
    tfa_token: String,
}

#[derive(serde::Deserialize, Debug)]
struct Login2faResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    status: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct OtpResponse {
    totp: String,
}

#[derive(serde::Deserialize)]
struct Config {
    email: String,
    password: String,
    max_loan_term: i32,
    min_interest: f32,
    #[serde(deserialize_with = "deserialize_url")]
    tfa_url: url::Url,
}

#[derive(Debug, serde::Deserialize)]
struct AccountInfo {
    #[serde(rename = "currencyIso")]
    currency_iso: String,
    #[serde(rename = "availableMoney", deserialize_with = "string_to_f64")]
    available_money: f64,
    #[serde(rename = "invested", deserialize_with = "string_to_f64")]
    invested: f64,
    #[serde(rename = "totalProfit", deserialize_with = "string_to_f64")]
    total_profit: f64,
    #[serde(rename = "totalBalance", deserialize_with = "string_to_f64")]
    total_balance: f64,
    #[serde(rename = "balanceGrowth", deserialize_with = "string_to_f64")]
    balance_growth: f64,
    #[serde(rename = "balanceGrowthAmount", deserialize_with = "string_to_f64")]
    balance_growth_amount: f64,
}

#[derive(serde::Deserialize)]
struct Loans {
    data: Vec<Loan>,
}

#[derive(serde::Deserialize)]
struct Loan {
    #[serde(rename = "loanId")]
    loan_id: i64,
    #[serde(rename = "availableToInvest")]
    available_to_invest: f64,
    #[serde(rename = "interestRate")]
    interest_rate: f32,
    #[serde(rename = "allowedToInvest")]
    allowed_to_invest: bool,
    term: i32,
}

#[derive(serde::Serialize)]
struct InvestmentPayload {
    amount: String,
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Url::parse(&s).map_err(serde::de::Error::custom)
}

fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

fn filter_desirable_loans(loans: Vec<Loan>, max_term: i32, min_interest_rate: f32) -> Vec<Loan> {
    let mut desirable_loans: Vec<Loan> = loans
        .into_iter()
        .filter(|loan| {
            loan.allowed_to_invest
                && loan.term <= max_term
                && loan.interest_rate >= min_interest_rate
        })
        .collect();

    desirable_loans.sort_by(|a, b| {
        b.interest_rate
            .partial_cmp(&a.interest_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    desirable_loans
}

async fn login(client: &reqwest::Client, email: &str, password: &str) -> Result<String> {
    let url = format!("{}/v1/investor/login", BASE_URL);

    let payload = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };
    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .context("Failed to send login request")?;

    if response.status().is_success() {
        let login_response: LoginResponse = response
            .json()
            .await
            .context("Failed to parse login response")?;
        if login_response.tfa_is_active {
            Ok(login_response.tfa_token)
        } else {
            Err(anyhow!("Two-factor authentication is not active"))
        }
    } else {
        Err(anyhow!(
            "Failed to log in. Status code: {}",
            response.status()
        ))
    }
}

async fn request_2fa(tfa_token: &str, tfa_url: url::Url) -> Result<String> {
    // Get the OTP from a local service
    let otp_response: OtpResponse = reqwest::get(tfa_url).await?.json().await?;

    // Prepare the payload for the 2FA request
    let payload = json!({
        "code": otp_response.totp,
        "tfa_token": tfa_token,
    });

    // Send the 2FA request
    let response = reqwest::Client::new()
        .post(format!("{}/v1/investor/login/2fa", BASE_URL))
        .json(&payload)
        .send()
        .await?;

    // Check if the request was successful
    let status = response.status();
    let bytes = response.bytes().await?;
    let raw_response = String::from_utf8_lossy(&bytes);

    if status != reqwest::StatusCode::OK {
        eprintln!(
            "Failed to login with 2FA. Status: {}, Response: {}",
            status, raw_response
        );
        return Err(anyhow::anyhow!("Failed to login with 2FA"));
    }

    let login_response: Login2faResponse = serde_json::from_str(&raw_response)?;
    Ok(login_response.access_token)
}

async fn invest_in_loan(
    client: &reqwest::Client,
    access_token: &str,
    loan_id: i64,
    investment_amount: f64,
) -> Result<()> {
    // Define the endpoint URL
    let url = format!("{}/v1/loans/{}", BASE_URL, loan_id);

    // Define the payload
    let payload = json!({
        "amount": format!("{:.2}", investment_amount),
    });

    info!(
        "Investing in loan: https://peerberry.com/en/client/loan/{}",
        loan_id
    );
    if env::var("DRY_RUN").is_ok() {
        info!(
            "DRY RUN: Would have invested {} in loan with ID {}",
            investment_amount, loan_id
        );
        Ok(())
    } else {
        // Make the POST request
        let response = client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully invested in loan {}", loan_id);
        } else {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body.".to_string());
            error!("Failed to invest in loan {}: {}", loan_id, error_body);
        }

        Ok(())
    }
}

fn read_config() -> Result<String> {
    // Try reading from current directory
    if let Ok(content) = fs::read_to_string("config.toml") {
        return Ok(content);
    }

    // If failed, try reading from XDG config directory
    if let Some(config_path) = xdg_config_path("peerberry", "config.toml") {
        fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file at {:?}", config_path))
    } else {
        Err(anyhow::anyhow!("Config file not found"))
    }
}

fn xdg_config_path(app_name: &str, file_name: &str) -> Option<std::path::PathBuf> {
    let xdg_dirs = BaseDirectories::with_prefix(app_name).ok()?;
    xdg_dirs.find_config_file(file_name)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let config_content = read_config()?;
    let config: Config = toml::from_str(&config_content)?;

    let client = reqwest::Client::new();

    let tfa_token = login(&client, &config.email, &config.password).await?;

    let access_token = request_2fa(&tfa_token, config.tfa_url).await?;

    // 1. Fetch balance
    let raw_response = client
        .get(&format!("{}/v2/investor/balance/main", BASE_URL))
        .bearer_auth(&access_token)
        .send()
        .await?
        .text()
        .await?;

    let account_info: AccountInfo = match serde_json::from_str(&raw_response) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Failed to deserialize account info: {}", e);
            eprintln!("Raw response: {}", raw_response);
            std::process::exit(1);
        }
    };

    info!("Available balance: {}", account_info.available_money);

    if account_info.available_money == 0.0 {
        info!("Insufficient balance to invest.");
        return Ok(());
    }

    // 2. Fetch loans
    let loans_url = format!("{}/v1/loans?sort=-loanId&offset=0&pageSize=40", BASE_URL);
    let loans: Loans = client
        .get(&loans_url)
        .bearer_auth(&access_token)
        .send()
        .await?
        .json()
        .await?;
    info!("Available loans: {}", loans.data.len());

    let state = std::sync::Arc::new(State {
        available_money: account_info.available_money,
        loans: loans.data,
    });

    // 3. Select loans to invest
    let desirable_loans =
        filter_desirable_loans(loans.data, config.max_loan_term, config.min_interest);
    info!("Desirable loans: {}", desirable_loans.len());

    // 4. Invest in the selected loans
    let mut available_money = account_info.available_money;
    for loan in &desirable_loans {
        let investment_amount = available_money.min(loan.available_to_invest);
        if investment_amount <= 0.0 {
            info!("No more funds to invest.");
            break;
        }
        if let Err(e) =
            invest_in_loan(&client, &access_token, loan.loan_id, investment_amount).await
        {
            eprintln!("Failed to invest in loan {}: {}", loan.loan_id, e);
        }
        available_money -= investment_amount;
    }

    Ok(())
}
