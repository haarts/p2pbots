extern crate reqwest;
extern crate serde;
// extern crate serde_derive;
extern crate toml;
use std::error::Error;
use url::Url;

use anyhow::Result;
use std::fs;

const BASE_URL: &str = "https://esketit.com/api/investor";

#[derive(serde::Deserialize)]
struct Config {
    username: String,
    password: String,
    max_term_period: u32,
    min_interest_rate: f32,
    #[serde(deserialize_with = "deserialize_url")]
    tfa_url: url::Url,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct OtpResponse {
    totp: String,
}

#[derive(serde::Serialize)]
struct LoginRequest {
    email: String,
    password: String,
    // ... other fields ...
}

#[derive(serde::Deserialize)]
struct LoginResponse {
    #[serde(rename = "investorNumber")]
    investor_number: String,
    // ... other fields ...
}

#[derive(serde::Serialize, Debug)]
struct TwoFactorAuthRequest {
    totp: String,
}

#[derive(serde::Serialize)]
struct AccountInfoRequest {
    #[serde(rename = "currencyCode")]
    currency_code: String,
}

#[derive(serde::Deserialize, Debug)]
struct AccountInfoResponse {
    #[serde(rename = "cashBalance")]
    cash_balance: f32,
    // ... other fields ...
}

#[derive(serde::Serialize)]
struct QueryInvestmentsRequest {
    page: u32,
    #[serde(rename = "pageSize")]
    page_size: u32,
    filter: Filter,
}

#[derive(serde::Serialize)]
struct Filter {
    #[serde(rename = "principalOfferFrom")]
    principal_offer_from: String,
    #[serde(rename = "currencyCode")]
    currency_code: String,
}

#[derive(serde::Deserialize, Debug)]
struct QueryInvestmentsResponse {
    items: Vec<Loan>,
}

#[derive(serde::Deserialize, Debug)]
struct Loan {
    #[serde(rename = "loanId")]
    loan_id: u64,
    #[serde(rename = "interestRatePercent")]
    interest_rate_percent: f32,
    // ... other fields ...
}

#[derive(serde::Serialize)]
struct InvestmentRequest {
    #[serde(rename = "loanId")]
    loan_id: u64,
    amount: String,
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Url::parse(&s).map_err(serde::de::Error::custom)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load and parse config
    let config_contents = fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_contents).unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(jar.clone())
        // .cookie_store(true)
        .default_headers(headers)
        .build()
        .unwrap();

    // 1. Login
    let login_request = LoginRequest {
        email: config.username,
        password: config.password,
        // ... other fields ...
    };
    let response = client
        .post(format!("{}/public/login", BASE_URL))
        .json(&login_request)
        .send()
        .await?;

    let bytes = response.bytes().await?;
    let raw_response = String::from_utf8_lossy(&bytes);

    let login_response: LoginResponse = serde_json::from_str(&raw_response)?;

    // 2. Supply 2FA token
    let otp_response: OtpResponse = reqwest::get(config.tfa_url).await?.json().await?;
    let two_factor_auth_request = TwoFactorAuthRequest {
        totp: otp_response.totp,
    };

    client
        .post(format!("{}/public/confirm-login", BASE_URL))
        .json(&two_factor_auth_request)
        .send()
        .await?
        .error_for_status()?;

    let response = client
        .get(format!("{}/profile", BASE_URL))
        .send()
        .await?
        .error_for_status()?;

    let mut token = String::new();
    for cookie in response.cookies() {
        if cookie.name() == "XSRF-TOKEN" {
            token = cookie.value().to_string();
        }
    }

    let bytes = response.bytes().await?;
    let raw_response = String::from_utf8_lossy(&bytes);
    println!("{}", raw_response);

    // 3. Get account information
    let account_info_request = AccountInfoRequest {
        currency_code: "EUR".to_string(),
    };

    // let serialized = serde_json::to_string(&account_info_request).unwrap();
    println!("{:?}", jar);
    let account_info_response: AccountInfoResponse = client
        .post(format!("{}/account-summary", BASE_URL))
        .header("X-XSRF-TOKEN", token.clone())
        .json(&account_info_request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    println!("{:?}", account_info_response);

    // 4. Query available investments
    let query_investments_request = QueryInvestmentsRequest {
        page: 1,
        page_size: 20,
        filter: Filter {
            principal_offer_from: "10".to_string(),
            currency_code: "EUR".to_string(),
        },
    };
    let response = client
        .post(format!("{}/public/query-primary-market", BASE_URL))
        .json(&query_investments_request)
        .send()
        .await?;
    // .error_for_status()?
    // .json()
    // .await?;
    let cookies = response.cookies();
    for cookie in cookies {
        if cookie.name() == "XSRF-TOKEN" {
            token = cookie.value().to_string();
        }
    }

    let bytes = response.bytes().await?;
    let query_investments_response: QueryInvestmentsResponse = serde_json::from_slice(&bytes)?;
    println!("{:?}", query_investments_response);

    // 5. Select investment with highest interest rate within criteria
    let mut highest_interest_rate = config.min_interest_rate;
    let mut selected_loan = None;
    for loan in query_investments_response.items {
        if loan.interest_rate_percent > highest_interest_rate {
            highest_interest_rate = loan.interest_rate_percent;
            selected_loan = Some(loan);
        }
    }

    if let Some(loan) = selected_loan {
        println!("{:?}", loan);
        // 6. Invest in selected loan
        let investment_request = InvestmentRequest {
            loan_id: loan.loan_id,
            amount: "5".to_string(), // Assume you want to invest €50, or calculate the amount based on your criteria
        };
        let investment_response = client
            .post(format!("{}/invest", BASE_URL))
            .header("X-XSRF-TOKEN", token)
            .json(&investment_request)
            .send()
            .await?;

        // Check the response to confirm the investment was successful
        if investment_response.status().is_success() {
            println!("Investment successful!");
        } else {
            eprintln!("Investment failed: {:?}", investment_response.text().await?);
        }
    }

    Ok(())
}
