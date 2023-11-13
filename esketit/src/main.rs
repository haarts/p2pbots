extern crate reqwest;
extern crate serde;
// extern crate serde_derive;
extern crate toml;
use std::error::Error;

use anyhow::Result;
use reqwest::Client;
// use serde_derive::{Deserialize, Serialize};
use std::fs;

#[derive(serde::Deserialize)]
struct Config {
    username: String,
    password: String,
    max_term_period: u32,
    min_interest_rate: f32,
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

#[derive(serde::Serialize)]
struct TwoFactorAuthRequest {
    totp: String,
}

#[derive(serde::Serialize)]
struct AccountInfoRequest {
    currency_code: String,
}

#[derive(serde::Deserialize)]
struct AccountInfoResponse {
    cash_balance: f32,
    // ... other fields ...
}

#[derive(serde::Serialize)]
struct QueryInvestmentsRequest {
    page: u32,
    page_size: u32,
    // filter: Filter,
}

#[derive(serde::Serialize)]
struct Filter {
    principal_offer_from: String,
    currency_code: String,
}

#[derive(serde::Deserialize)]
struct QueryInvestmentsResponse {
    items: Vec<Loan>,
}

#[derive(serde::Deserialize)]
struct Loan {
    loan_id: u64,
    interest_rate_percent: f32,
    // ... other fields ...
}

#[derive(serde::Serialize)]
struct InvestmentRequest {
    loan_id: u64,
    amount: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load and parse config
    let config_contents = fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_contents).unwrap();

    let client = reqwest::Client::builder()
        // Enable cookie store
        .cookie_store(true)
        .build()
        .unwrap();

    // 1. Login
    let login_request = LoginRequest {
        email: config.username,
        password: config.password,
        // ... other fields ...
    };
    // let login_response: LoginResponse = client
    let response = client
        .post("https://esketit.com/api/investor/public/login")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&login_request)
        .send()
        .await?;
    // for cookie in response.cookies() {
    //     println!("Cookie: {:?}\n", cookie);
    //     println!("name: {:?}\n", cookie.name());
    // }

    let bytes = response.bytes().await?;
    let raw_response = String::from_utf8_lossy(&bytes);
    println!("{}", raw_response);

    let login_response: LoginResponse = serde_json::from_str(&raw_response)?;

    // 2. Supply 2FA token
    // Assume totp is obtained some other way, e.g., user input
    let totp = "613023";
    let two_factor_auth_request = TwoFactorAuthRequest {
        totp: totp.to_string(),
    };

    let totp_client = Client::new();
    totp_client
        .post("https://esketit.com/api/investor/public/confirm-login")
        .json(&two_factor_auth_request)
        .send()
        .await?;

    // 3. Get account information
    let account_info_request = AccountInfoRequest {
        currency_code: "EUR".to_string(),
    };
    let account_info_response: AccountInfoResponse = client
        .post("https://esketit.com/api/investor/account-summary")
        .json(&account_info_request)
        .send()
        .await?
        // .error_for_status()
        .json()
        .await?;

    // 4. Query available investments
    let query_investments_request = QueryInvestmentsRequest {
        page: 1,
        page_size: 20,
        // filter: Filter {
        // principal_offer_from: "10".to_string(),
        // currency_code: "EUR".to_string(),
        // },
    };
    let query_investments_response: QueryInvestmentsResponse = client
        .post("https://esketit.com/api/investor/public/query-primary-market")
        .json(&query_investments_request)
        .send()
        .await?
        .json()
        .await?;

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
        // 6. Invest in selected loan
        let investment_request = InvestmentRequest {
            loan_id: loan.loan_id,
            amount: 50.0, // Assume you want to invest €50, or calculate the amount based on your criteria
        };
        let investment_response = client
            .post("https://esketit.com/api/investor/invest")
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
