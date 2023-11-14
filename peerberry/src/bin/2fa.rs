use axum::{http::StatusCode, routing::get, Json, Router};
use clap::Parser;
use serde::Serialize;
use totp_rs::TOTP;

#[derive(clap::Parser, Debug)]
struct Args {
    host: std::net::SocketAddr,
}

#[derive(Debug, Serialize)]
struct TOTPResponse {
    totp: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

async fn generate_totp(
    totp: TOTP,
) -> Result<Json<TOTPResponse>, (StatusCode, Json<ErrorResponse>)> {
    match totp.generate_current() {
        Ok(current_totp) => Ok(Json(TOTPResponse { totp: current_totp })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to generate TOTP".to_string(),
            }),
        )),
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Build our application with a single route
    let app = Router::new()
        .route(
            "/peerberry", 
            get({
            let otpauth_url =
                "otpauth://totp/Peerberry:harm%40aarts.email?secret=O4SGUC3YX7RHLNT3BCTRQWPTTZXW7NG2&issuer=Peerberry";
            move || generate_totp(TOTP::from_url(&otpauth_url).unwrap())
        }),
        )
        .route(
            "/esketit", 
            get({
            let otpauth_url =
                "otpauth://totp/Esketit: harm@aarts.email?secret=XRREDOFU5AVMRHVW&digits=6";
            move || generate_totp(TOTP::from_url_unchecked(&otpauth_url).unwrap())
        }));

    println!("Listening on {}", args.host);
    axum::Server::bind(&args.host)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
