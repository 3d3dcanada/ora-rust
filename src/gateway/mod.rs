//! OrA Gateway Module
//!
//! HTTP + WebSocket server for OrA.

pub mod http;
pub mod mcp;
pub mod websocket;

pub use http::create_router;
pub use websocket::websocket_handler;
