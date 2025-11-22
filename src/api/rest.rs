// REST API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::routes::{parser, lookup::RouteEngine, RoutingTable};
use super::{TraceRouteRequest, TraceRouteResponse, ErrorResponse};

#[derive(Clone)]
pub struct AppState {
    // Shared application state
}

impl AppState {
    pub fn new() -> Self {
        AppState {}
    }
}

pub fn create_api_router() -> Router {
    let state = Arc::new(AppState::new());

    Router::new()
        .route("/api/routing-table", get(get_routing_table))
        .route("/api/trace-route", post(trace_route))
        .with_state(state)
}

async fn get_routing_table(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<RoutingTable>, (StatusCode, Json<ErrorResponse>)> {
    match parser::get_routing_table() {
        Ok(table) => Ok(Json(table)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "FailedToGetRoutingTable".to_string(),
                message: e,
            }),
        )),
    }
}

async fn trace_route(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<TraceRouteRequest>,
) -> Result<Json<TraceRouteResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Resolve destination to IP
    let ip = match request.destination.parse() {
        Ok(ip) => ip,
        Err(_) => {
            // Try DNS resolution
            match tokio::net::lookup_host(format!("{}:0", request.destination))
                .await
                .ok()
                .and_then(|mut addrs| addrs.next())
            {
                Some(addr) => addr.ip(),
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "InvalidDestination".to_string(),
                            message: format!("Could not resolve destination: {}", request.destination),
                        }),
                    ));
                }
            }
        }
    };

    // Get routing table
    let routing_table = match parser::get_routing_table() {
        Ok(table) => table,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "FailedToGetRoutingTable".to_string(),
                    message: e,
                }),
            ));
        }
    };

    // Perform route lookup
    let engine = RouteEngine::new(&routing_table);
    let matched_route = engine.lookup(ip).cloned();

    if matched_route.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NoRouteToHost".to_string(),
                message: format!("No route found to {}", ip),
            }),
        ));
    }

    Ok(Json(TraceRouteResponse {
        destination: request.destination,
        resolved_ip: ip.to_string(),
        matched_route,
    }))
}
