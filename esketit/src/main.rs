use reqwest;
use serde;
use std::error::Error;
use toml;
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
struct QueryLoansRequest {
    page: u32,
    #[serde(rename = "pageSize")]
    page_size: u32,
    filter: LoansFilter,
    #[serde(rename = "sortBy")]
    sort_by: String,
}

#[derive(serde::Serialize)]
struct LoansFilter {
    #[serde(rename = "principalOfferFrom")]
    principal_offer_from: String,
    #[serde(rename = "currencyCode")]
    currency_code: String,
}

#[derive(serde::Deserialize, Debug)]
struct QueryLoansResponse {
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
struct QueryInvestmentsRequest {
    page: u32,
    #[serde(rename = "pageSize")]
    page_size: u32,
    filter: InvestmentsFilter,
    #[serde(rename = "sortBy")]
    sort_by: String,
}

#[derive(serde::Serialize)]
struct InvestmentsFilter {
    #[serde(rename = "currencyCode")]
    currency_code: String,
    #[serde(
        rename = "smDiscountOrPremiumPercentFrom",
        skip_serializing_if = "Option::is_none"
    )]
    sm_discount_or_premium_percent_from: Option<String>,
    #[serde(
        rename = "smDiscountOrPremiumPercentTo",
        skip_serializing_if = "Option::is_none"
    )]
    sm_discount_or_premium_percent_to: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct QueryInvestmentsResponse {
    items: Vec<Investment>,
}

#[derive(serde::Deserialize, Debug)]
struct Investment {
    #[serde(rename = "investmentId")]
    investment_id: u64,
    #[serde(rename = "interestRatePercent")]
    interest_rate_percent: f32,
    #[serde(rename = "termInDays")]
    term_in_days: i32,
    #[serde(rename = "smDiscountOrPremiumPercent")]
    sm_discount_or_premium_percent: f32,
    #[serde(rename = "originatorId")]
    originator_id: u64,
    #[serde(rename = "smPrice")]
    sm_price: f32,
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

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .default_headers(headers)
        .build()
        .unwrap();

    // 1. Login, this sets a bunch of cookies
    let login_request = LoginRequest {
        email: config.username,
        password: config.password,
    };
    client
        .post(format!("{}/public/login", BASE_URL))
        .json(&login_request)
        .send()
        .await?;

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

    // 3. Get account information
    let account_info_request = AccountInfoRequest {
        currency_code: "EUR".to_string(),
    };

    let account_info_response: AccountInfoResponse = client
        .post(format!("{}/account-summary", BASE_URL))
        .header("X-XSRF-TOKEN", token.clone())
        .json(&account_info_request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if account_info_response.cash_balance < 5.0 {
        println!(
            "Not enough cash to invest. Currently available: {}",
            account_info_response.cash_balance
        );
        return Ok(());
    }

    // 4. Query available loans
    let query_investments_request = QueryLoansRequest {
        page: 1,
        page_size: 20,
        sort_by: "interestRatePercent".to_string(),
        filter: LoansFilter {
            principal_offer_from: "5".to_string(),
            currency_code: "EUR".to_string(),
        },
    };
    let response = client
        .post(format!("{}/public/query-primary-market", BASE_URL))
        .json(&query_investments_request)
        .send()
        .await?
        .error_for_status()?;

    let cookies = response.cookies();
    for cookie in cookies {
        if cookie.name() == "XSRF-TOKEN" {
            token = cookie.value().to_string();
        }
    }

    let bytes = response.bytes().await?;
    let query_loans_response: QueryLoansResponse = serde_json::from_slice(&bytes)?;

    // 5. Query available investments
    let secondary_market_query_investments_request = QueryInvestmentsRequest {
        page: 1,
        page_size: 20,
        sort_by: "smDiscountOrPremiumPercent".to_string(),
        filter: InvestmentsFilter {
            currency_code: "EUR".to_string(),
            sm_discount_or_premium_percent_from: Some("-2.0".to_string()),
            sm_discount_or_premium_percent_to: Some("-0.5".to_string()),
        },
    };
    println!(
        "{:?}",
        serde_json::to_string(&secondary_market_query_investments_request)
    );
    let response = client
        .post(format!("{}/public/query-secondary-market", BASE_URL))
        .json(&secondary_market_query_investments_request)
        .send()
        .await?
        .error_for_status()?;

    let cookies = response.cookies();
    for cookie in cookies {
        if cookie.name() == "XSRF-TOKEN" {
            token = cookie.value().to_string();
        }
    }

    let bytes = response.bytes().await?;
    let query_investments_response: QueryInvestmentsResponse = serde_json::from_slice(&bytes)?;
    println!("secondary: {:?}", query_investments_response);

    // 6. Select investment with highest interest rate within criteria
    let mut highest_interest_rate = config.min_interest_rate;
    let mut selected_loan = None;
    for loan in query_loans_response.items {
        if loan.interest_rate_percent > highest_interest_rate {
            highest_interest_rate = loan.interest_rate_percent;
            selected_loan = Some(loan);
        }
    }

    if let Some(loan) = selected_loan {
        // 7. Invest in selected loan
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
