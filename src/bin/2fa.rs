extern crate axum;
extern crate serde_json;
extern crate totp_rs;

use axum::{http::StatusCode, routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use totp_rs::TOTP;

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
                "otpauth://totp/Peerberry:harm%40aarts.email?secret=O4SGUC3YX7RHLNT3BCTRQWPTTZXW7NG2&issuer=Peerberry";
            move || generate_totp(TOTP::from_url(&otpauth_url).unwrap())
        }));

    // Run our app with hyper on localhost:3030
    let addr = SocketAddr::from(([100, 112, 251, 5], 3030));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
