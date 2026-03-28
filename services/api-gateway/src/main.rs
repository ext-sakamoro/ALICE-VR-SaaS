// ALICE-VR-SaaS api-gateway
// License: AGPL-3.0-or-later

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{any, get},
    Router,
};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

const CORE_ENGINE_URL: &str = "http://127.0.0.1:9146";

#[derive(Clone)]
struct GatewayState {
    client: reqwest::Client,
    rate_limits: Arc<DashMap<String, u64>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    upstream: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "alice-vr-api-gateway",
        version: "0.1.0",
        upstream: CORE_ENGINE_URL,
    })
}

async fn proxy(
    State(state): State<GatewayState>,
    req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target = format!("{}{}{}", CORE_ENGINE_URL, path, query);

    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    info!("proxy {} -> {}", method, target);

    let upstream_resp = state
        .client
        .request(method, &target)
        .body(body_bytes)
        .header("content-type", "application/json")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("upstream error: {}", e);
            StatusCode::BAD_GATEWAY
        })?;

    let status = StatusCode::from_u16(upstream_resp.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = upstream_resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok((status, body).into_response())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let state = GatewayState {
        client: reqwest::Client::new(),
        rate_limits: Arc::new(DashMap::new()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/vr/{*path}", any(proxy))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8146").await.unwrap();
    info!("alice-vr-api-gateway listening on :8146 -> core-engine :9146");
    axum::serve(listener, app).await.unwrap();
}
