// API module - REST endpoints and WebSocket handling

pub mod rest;
pub mod websocket;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceRouteRequest {
    pub destination: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceRouteResponse {
    pub destination: String,
    pub resolved_ip: String,
    pub matched_route: Option<crate::routes::Route>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracerouteRequest {
    pub destination: String,
}

pub use crate::discovery::traceroute::TracerouteResult;
