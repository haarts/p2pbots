use axum::{http::StatusCode, routing::get, Json, Router};
use clap::Parser;
use serde::Serialize;
use totp_rs::TOTP;

#[derive(clap::Parser, Debug)]
struct Args {
    host: std::net::SocketAddr,
    esketit: String,
    peerberry: String,
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

    let esketit = totp_rs::TOTP::new_unchecked(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        totp_rs::Secret::Encoded(args.esketit).to_bytes().unwrap(),
        None,
        "".to_string(),
    );

    let peerberry = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        totp_rs::Secret::Encoded(args.peerberry).to_bytes().unwrap(),
        None,
        "".to_string(),
    )
    .unwrap();

    let app = Router::new()
        .route("/peerberry", get(move || generate_totp(peerberry)))
        .route("/esketit", get(move || generate_totp(esketit)));

    println!("Listening on {}", args.host);
    axum::Server::bind(&args.host)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
