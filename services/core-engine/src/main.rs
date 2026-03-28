// ALICE-VR-SaaS core-engine
// License: AGPL-3.0-or-later

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize)]
struct Stats {
    total_requests: u64,
    session_requests: u64,
    tracking_requests: u64,
    render_requests: u64,
    comfort_requests: u64,
}

#[derive(Clone)]
struct AppState {
    stats: Arc<Mutex<Stats>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

#[derive(Deserialize)]
struct SessionRequest {
    user_id: String,
    headset_model: Option<String>,
    scene_id: Option<String>,
}

#[derive(Serialize)]
struct SessionResponse {
    id: String,
    user_id: String,
    headset_model: String,
    scene_id: String,
    status: &'static str,
    created_at: &'static str,
}

#[derive(Deserialize)]
struct TrackingData {
    session_id: String,
    head_position: [f64; 3],
    head_rotation: [f64; 4],
    hand_left: Option<[f64; 3]>,
    hand_right: Option<[f64; 3]>,
}

#[derive(Serialize)]
struct TrackingResponse {
    id: String,
    session_id: String,
    latency_ms: f64,
    prediction_ms: f64,
    status: &'static str,
}

#[derive(Deserialize)]
struct RenderRequest {
    session_id: String,
    resolution_w: u32,
    resolution_h: u32,
    target_fps: Option<u32>,
}

#[derive(Serialize)]
struct RenderResponse {
    id: String,
    session_id: String,
    frame_time_ms: f64,
    gpu_utilization: f64,
    reprojection_active: bool,
}

#[derive(Serialize)]
struct ComfortResponse {
    session_id: &'static str,
    motion_sickness_risk: f64,
    locomotion_speed_ms: f64,
    ipd_mm: f64,
    recommendations: Vec<&'static str>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "alice-vr-core-engine",
        version: "0.1.0",
    })
}

async fn vr_session(
    State(state): State<AppState>,
    Json(req): Json<SessionRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.session_requests += 1;
    info!("vr/session user_id={}", req.user_id);
    Ok(Json(SessionResponse {
        id: Uuid::new_v4().to_string(),
        user_id: req.user_id,
        headset_model: req.headset_model.unwrap_or_else(|| "generic".to_string()),
        scene_id: req.scene_id.unwrap_or_else(|| "default".to_string()),
        status: "active",
        created_at: "2026-03-09T00:00:00Z",
    }))
}

async fn vr_tracking(
    State(state): State<AppState>,
    Json(req): Json<TrackingData>,
) -> Result<Json<TrackingResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.tracking_requests += 1;
    info!("vr/tracking session_id={}", req.session_id);
    Ok(Json(TrackingResponse {
        id: Uuid::new_v4().to_string(),
        session_id: req.session_id,
        latency_ms: 3.2,
        prediction_ms: 15.0,
        status: "ok",
    }))
}

async fn vr_render(
    State(state): State<AppState>,
    Json(req): Json<RenderRequest>,
) -> Result<Json<RenderResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.render_requests += 1;
    let fps = req.target_fps.unwrap_or(90);
    let frame_time = 1000.0 / fps as f64;
    info!("vr/render session_id={} {}x{} @{}fps", req.session_id, req.resolution_w, req.resolution_h, fps);
    Ok(Json(RenderResponse {
        id: Uuid::new_v4().to_string(),
        session_id: req.session_id,
        frame_time_ms: frame_time,
        gpu_utilization: 0.72,
        reprojection_active: false,
    }))
}

async fn vr_comfort(State(state): State<AppState>) -> Json<ComfortResponse> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.comfort_requests += 1;
    Json(ComfortResponse {
        session_id: "latest",
        motion_sickness_risk: 0.12,
        locomotion_speed_ms: 1.4,
        ipd_mm: 63.5,
        recommendations: vec![
            "locomotion_speed_within_comfort_zone",
            "ipd_calibrated",
        ],
    })
}

async fn vr_stats(State(state): State<AppState>) -> Json<Stats> {
    let stats = state.stats.lock().unwrap().clone();
    Json(stats)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let state = AppState {
        stats: Arc::new(Mutex::new(Stats::default())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/vr/session",  post(vr_session))
        .route("/api/v1/vr/tracking", post(vr_tracking))
        .route("/api/v1/vr/render",   post(vr_render))
        .route("/api/v1/vr/comfort",  get(vr_comfort))
        .route("/api/v1/vr/stats",    get(vr_stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9146").await.unwrap();
    info!("alice-vr-core-engine listening on :9146");
    axum::serve(listener, app).await.unwrap();
}
