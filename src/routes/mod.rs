// Routes module - handles routing table parsing and route lookups

pub mod lookup;
pub mod parser;

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub destination: String, // CIDR notation
    pub gateway: Option<IpAddr>,
    pub interface: String,
    pub metric: u32,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTable {
    pub hostname: String,
    pub routes: Vec<Route>,
    pub timestamp: String,
}
