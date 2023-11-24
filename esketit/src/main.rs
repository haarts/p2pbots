use reqwest;
use serde;
use toml;
use url::Url;

use anyhow::anyhow;
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
}

#[derive(serde::Deserialize)]
struct LoginResponse {
    #[serde(rename = "investorNumber")]
    investor_number: String,
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

#[derive(serde::Deserialize, Debug)]
struct PortfolioResponse {
    items: Vec<CurrentInvestment>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CurrentInvestment {
    #[serde(rename = "investmentId")]
    investment_id: u64,
    #[serde(rename = "loanId")]
    loan_id: u64,
    #[serde(rename = "interestRatePercent")]
    interest_rate_percent: f64,
    #[serde(rename = "investmentDate")]
    investment_date: String,
    #[serde(rename = "issueDate")]
    issue_date: String,
    #[serde(rename = "maturityDate")]
    maturity_date: String,
    #[serde(rename = "nextPaymentDate")]
    next_payment_date: String,
    #[serde(rename = "termInDays")]
    term_in_days: i32,
    #[serde(rename = "totalPayments")]
    total_payments: u32,
    #[serde(rename = "openPayments")]
    open_payments: u32,
    #[serde(rename = "closedPayments")]
    closed_payments: u32,
    #[serde(rename = "originatorCompanyName")]
    originator_company_name: String,
    #[serde(rename = "originatorId")]
    originator_id: u64,
    #[serde(rename = "productCode")]
    product_code: String,
    #[serde(rename = "productLabel")]
    product_label: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    #[serde(rename = "collectionStatus")]
    collection_status: String,
    closed: bool,
    #[serde(rename = "principalInvested")]
    principal_invested: f64,
    #[serde(rename = "principalOutstanding")]
    principal_outstanding: f64,
    #[serde(rename = "principalPaid")]
    principal_paid: f64,
    #[serde(rename = "principalPending")]
    principal_pending: f64,
    #[serde(rename = "principalReceived")]
    principal_received: f64,
    #[serde(rename = "interestPaid")]
    interest_paid: f64,
    #[serde(rename = "interestBonusPaid")]
    interest_bonus_paid: f64,
    #[serde(rename = "interestPending")]
    interest_pending: f64,
    #[serde(rename = "interestReceived")]
    interest_received: f64,
    #[serde(rename = "bonusPaid")]
    bonus_paid: f64,
    #[serde(rename = "bonusPending")]
    bonus_pending: f64,
    #[serde(rename = "bonusReceived")]
    bonus_received: f64,
    #[serde(rename = "totalPending")]
    total_pending: f64,
    #[serde(rename = "smOfferPrincipalAvailable")]
    sm_offer_principal_available: f64,
    #[serde(rename = "smPrincipalSold")]
    sm_principal_sold: f64,
    #[serde(rename = "smDiscountOrPremiumPercent")]
    sm_discount_or_premium_percent: Option<f64>,
    #[serde(rename = "currencyCode")]
    currency_code: String,
    #[serde(rename = "currencySymbol")]
    currency_symbol: String,
    #[serde(rename = "agreementFileName")]
    agreement_file_name: String,
    #[serde(rename = "agreementFileReference")]
    agreement_file_reference: String,
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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Loan {
    #[serde(rename = "loanId")]
    loan_id: i64,
    #[serde(rename = "issueDate")]
    issue_date: String,
    #[serde(rename = "interestRatePercent")]
    interest_rate_percent: f32,
    #[serde(rename = "principalIssued")]
    principal_issued: f64,
    #[serde(rename = "principalOffer")]
    principal_offer: f64,
    #[serde(rename = "principalOutstanding")]
    principal_outstanding: f64,
    #[serde(rename = "currencyCode")]
    currency_code: String,
    #[serde(rename = "currencySymbol")]
    currency_symbol: String,
    #[serde(rename = "totalPayments")]
    total_payments: i32,
    #[serde(rename = "openPayments")]
    open_payments: i32,
    #[serde(rename = "closedPayments")]
    closed_payments: i32,
    #[serde(rename = "maturityDate")]
    maturity_date: String,
    #[serde(rename = "nextPaymentDate")]
    next_payment_date: String,
    #[serde(rename = "termInDays")]
    term_in_days: i32,
    #[serde(rename = "originatorCompanyName")]
    originator_company_name: String,
    #[serde(rename = "originatorId")]
    originator_id: i64,
    #[serde(rename = "productCode")]
    product_code: String,
    #[serde(rename = "productLabel")]
    product_label: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    #[serde(rename = "hasBuyback")]
    has_buyback: bool,
    extensions: i32,
    #[serde(rename = "extendedForDays")]
    extended_for_days: i32,
    #[serde(rename = "myInvestments")]
    my_investments: f64,
    #[serde(rename = "myInvestmentsPercent")]
    my_investments_percent: f64,
    #[serde(rename = "fundedPercent")]
    funded_percent: i32,
    #[serde(rename = "amountFunded")]
    amount_funded: f64,
    #[serde(rename = "amountAvailable")]
    amount_available: f64,
    #[serde(rename = "availablePercent")]
    available_percent: i32,
    #[serde(rename = "loanStatus")]
    loan_status: String,
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Investment {
    #[serde(rename = "investmentId")]
    investment_id: i64,
    #[serde(rename = "loanId")]
    loan_id: i64,
    #[serde(rename = "issueDate")]
    issue_date: String,
    #[serde(rename = "interestRatePercent")]
    interest_rate_percent: f32,
    #[serde(rename = "currencyCode")]
    currency_code: String,
    #[serde(rename = "currencySymbol")]
    currency_symbol: String,
    #[serde(rename = "totalPayments")]
    total_payments: i32,
    #[serde(rename = "openPayments")]
    open_payments: i32,
    #[serde(rename = "closedPayments")]
    closed_payments: i32,
    #[serde(rename = "maturityDate")]
    maturity_date: String,
    #[serde(rename = "nextPaymentDate")]
    next_payment_date: String,
    #[serde(rename = "termInDays")]
    term_in_days: i32,
    #[serde(rename = "originatorCompanyName")]
    originator_company_name: String,
    #[serde(rename = "originatorId")]
    originator_id: i64,
    #[serde(rename = "productCode")]
    product_code: String,
    #[serde(rename = "productLabel")]
    product_label: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    #[serde(rename = "collectionStatus")]
    collection_status: String,
    #[serde(rename = "smOfferPrincipalAvailable")]
    sm_offer_principal_available: f64,
    #[serde(rename = "smDiscountOrPremiumPercent")]
    sm_discount_or_premium_percent: f64,
    #[serde(rename = "smPrice")]
    sm_price: f64,
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

struct Client {
    client: reqwest::Client,
    xsrf_token: String,
}

struct State {
    portfolio: Vec<CurrentInvestment>,
    available_loans: Vec<Loan>,
    available_investments: Vec<Investment>,
    cash_balance: f32,
}

impl Client {
    pub fn new() -> anyhow::Result<Client> {
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
            .build()?;

        return Ok(Client {
            client,
            xsrf_token: String::new(),
        });
    }

    async fn fetch_remote_state(&mut self) -> anyhow::Result<State> {
        // Load and parse config
        let config_contents = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_contents)?;

        // 1. Login, this sets a bunch of cookies
        let login_request = LoginRequest {
            email: config.username,
            password: config.password,
        };
        self.client
            .post(format!("{}/public/login", BASE_URL))
            .json(&login_request)
            .send()
            .await?;

        // 2. Supply 2FA token
        let otp_response: OtpResponse = reqwest::get(config.tfa_url).await?.json().await?;
        let two_factor_auth_request = TwoFactorAuthRequest {
            totp: otp_response.totp,
        };

        self.client
            .post(format!("{}/public/confirm-login", BASE_URL))
            .json(&two_factor_auth_request)
            .send()
            .await?
            .error_for_status()?;

        let response = self
            .client
            .get(format!("{}/profile", BASE_URL))
            .send()
            .await?
            .error_for_status()?;

        for cookie in response.cookies() {
            if cookie.name() == "XSRF-TOKEN" {
                self.xsrf_token = cookie.value().to_string();
            }
        }

        let bytes = response.bytes().await?;
        let raw_response = String::from_utf8_lossy(&bytes);

        // 3. Get account information
        let account_info_request = AccountInfoRequest {
            currency_code: "EUR".to_string(),
        };

        let account_info_response: AccountInfoResponse = self
            .client
            .post(format!("{}/account-summary", BASE_URL))
            .header("X-XSRF-TOKEN", self.xsrf_token.clone())
            .json(&account_info_request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        if account_info_response.cash_balance < 5.0 {
            return Err(anyhow!(
                "Not enough cash to invest. Currently available: {}",
                account_info_response.cash_balance
            ));
        }
        println!("Enough money to invest. Continuing...");

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
        let response = self
            .client
            .post(format!("{}/public/query-primary-market", BASE_URL))
            .json(&query_investments_request)
            .send()
            .await?
            .error_for_status()?;

        let cookies = response.cookies();
        for cookie in cookies {
            if cookie.name() == "XSRF-TOKEN" {
                self.xsrf_token = cookie.value().to_string();
            }
        }

        let bytes = response.bytes().await?;
        let query_loans_response: QueryLoansResponse = serde_json::from_slice(&bytes)?;

        println!(
            "Found {} available loans on primary market",
            query_loans_response.items.len()
        );

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
        let response = self
            .client
            .post(format!("{}/public/query-secondary-market", BASE_URL))
            .json(&secondary_market_query_investments_request)
            .send()
            .await?
            .error_for_status()?;

        let cookies = response.cookies();
        for cookie in cookies {
            if cookie.name() == "XSRF-TOKEN" {
                self.xsrf_token = cookie.value().to_string();
            }
        }

        let bytes = response.bytes().await?;
        let query_investments_response: QueryInvestmentsResponse = serde_json::from_slice(&bytes)?;

        println!(
            "Found {} available loans on secondary market",
            query_investments_response.items.len()
        );

        //6. Query portfolio
        let response = self
            .client
            .post(format!("{}/query-my-investments", BASE_URL))
            .header("X-XSRF-TOKEN", self.xsrf_token.clone())
            .json(&serde_json::json!({
                "page": 1,
                "pageSize": 50,
                "filter": {
                    "showActive": true,
                    "showClosed": false,
                    "currencyCode": "EUR"
                }
            }))
            .send()
            .await?
            .error_for_status()?;

        let cookies = response.cookies();
        for cookie in cookies {
            if cookie.name() == "XSRF-TOKEN" {
                self.xsrf_token = cookie.value().to_string();
            }
        }

        let bytes = response.bytes().await?;
        let portfolio: PortfolioResponse = serde_json::from_slice(&bytes)?;

        println!(
            "Current portfolio contains {} investments",
            query_investments_response.items.len()
        );

        return Ok(State {
            cash_balance: account_info_response.cash_balance,
            portfolio: portfolio.items,
            available_investments: query_investments_response.items,
            available_loans: query_loans_response.items,
        });
    }

    async fn invest_loan(&mut self, loan_id: u64, amount: f32) -> anyhow::Result<()> {
        let investment_request = InvestmentRequest {
            loan_id,
            amount: amount.to_string(),
        };
        let investment_response = self
            .client
            .post(format!("{}/invest", BASE_URL))
            .header("X-XSRF-TOKEN", self.xsrf_token.clone())
            .json(&investment_request)
            .send()
            .await?;

        if investment_response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!(
                "Loan failed: {:?}",
                investment_response.text().await?
            ))
        }
    }
}

#[derive(serde::Deserialize)]
struct Accept {
    id: u64,
    amount: f32,
}

async fn shutdown(
    shutdown_signal: axum::extract::Extension<std::sync::Arc<std::sync::atomic::AtomicBool>>,
) {
    shutdown_signal
        .0
        .store(true, std::sync::atomic::Ordering::SeqCst);
}

#[axum_macros::debug_handler]
async fn accept_loan(
    axum::extract::Extension(client): axum::extract::Extension<
        std::sync::Arc<tokio::sync::Mutex<Client>>,
    >,
    axum::extract::Json(payload): axum::extract::Json<Accept>,
) -> Result<impl axum::response::IntoResponse, impl axum::response::IntoResponse> {
    let mut client = client.lock().await;
    client
        .invest_loan(payload.id, payload.amount)
        .await
        .map(|_| (axum::http::StatusCode::OK, "Loan accepted".to_string()))
        .map_err(|e| {
            let error_message = format!("Internal server error: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error_message)
        })
}

async fn accept_investment() {}

fn write_to_csv<T: serde::Serialize>(file_name: String, items: Vec<T>) -> anyhow::Result<()> {
    let mut wtr =
        csv::Writer::from_writer(std::io::BufWriter::new(std::fs::File::create(file_name)?));

    for item in items {
        wtr.serialize(item)?;
    }

    wtr.flush()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new()?;
    let state = client.fetch_remote_state().await?;

    let shared_client = std::sync::Arc::new(tokio::sync::Mutex::new(client));

    let shutdown_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let app = axum::Router::new()
        .route(
            "/shutdown",
            axum::routing::post(shutdown).layer(axum::extract::Extension(shutdown_signal.clone())),
        )
        .route(
            "/loans",
            axum::routing::post(accept_loan).layer(axum::extract::Extension(shared_client.clone())),
        )
        .route(
            "/investment",
            axum::routing::post(accept_investment).layer(axum::extract::Extension(shared_client)),
        );

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    let graceful = server.with_graceful_shutdown(async {
        loop {
            if shutdown_signal.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            // Check for shutdown signal every 1000ms
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });

    let now = chrono::Local::now();
    let format = chrono::format::strftime::StrftimeItems::new("%Y-%m-%d_%H-%M");
    let time_string = now.format_with_items(format).to_string();

    let loans = state.available_loans;
    let investments = state.available_investments;
    let portfolio = state.portfolio;
    write_to_csv(format!("{}_loans.csv", time_string), loans)?;
    write_to_csv(format!("{}_investments.csv", time_string), investments)?;
    write_to_csv(format!("{}_portfolio.csv", time_string), portfolio)?;
    // std::process::Command::new(format!(
    //     "wintermute --cash {} --prefix {}",
    //     state.cash_balance, time_string
    // ))
    // .spawn()?;

    graceful.await?;
    // let mut highest_interest_rate = config.min_interest_rate;
    // let mut selected_loan = None;
    // for loan in query_loans_response.items {
    //     if loan.interest_rate_percent > highest_interest_rate {
    //         highest_interest_rate = loan.interest_rate_percent;
    //         selected_loan = Some(loan);
    //     }
    // }

    // if let Some(loan) = selected_loan {
    //     // Check the response to confirm the investment was successful
    // }

    Ok(())
}
