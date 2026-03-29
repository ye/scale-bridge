//! HTTPS REST server for scale-bridge.
//!
//! Exposes scale readings over HTTP/JSON so web applications can query
//! scales without a direct serial connection.
//!
//! # Planned REST API
//!
//! ```text
//! GET  /api/weight        → 200 WeightReading as JSON
//! GET  /api/status        → 200 ScaleStatus as JSON
//! POST /api/zero          → 204 No Content
//! POST /api/tare          → 204 No Content
//! GET  /api/about         → 200 AboutInfo as JSON  (7600 series)
//! GET  /api/diagnostic    → 200 DiagnosticInfo as JSON  (7600 series)
//! ```
//!
//! # Planned CLI usage
//!
//! ```bash
//! # Serial-connected scale exposed over HTTPS
//! scale-bridge serve \
//!     --port 8443 \
//!     --scale-port /dev/ttyUSB0 \
//!     --cert cert.pem \
//!     --key key.pem
//!
//! # TCP/Ethernet scale exposed over HTTPS
//! scale-bridge serve \
//!     --port 8443 \
//!     --host 192.168.1.50 --tcp-port 3001 \
//!     --cert cert.pem --key key.pem
//! ```
//!
//! # Planned implementation notes
//!
//! - HTTP framework: `axum` (async, but server owns the sync scale connection
//!   in a `Mutex<Scale<...>>` behind a `tokio::task::spawn_blocking` wrapper)
//! - TLS: `rustls` via `axum-server` with `rustls` feature
//! - Systemd `Type=notify`: notify readiness via `NOTIFY_SOCKET` using the
//!   `sd-notify` crate once the listener is bound
//! - Graceful SIGTERM: `tokio::signal::ctrl_c()` and `axum::Server::with_graceful_shutdown`
//! - Socket activation: accept `TcpListener` from `SD_LISTEN_FDS` via
//!   `std::net::TcpListener::from_raw_fd()` for `Type=socket` systemd units

/// Server configuration.
pub struct ServerConfig {
    pub https_port: u16,
    pub scale_serial_port: Option<String>,
    pub scale_host: Option<String>,
    pub scale_tcp_port: u16,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

/// Start the HTTPS server.
///
/// # Errors
///
/// Returns an error if the server could not bind or TLS certificates are invalid.
///
/// # Panics
///
/// This function is not yet implemented and will panic if called.
pub fn serve(_config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    todo!(
        "HTTPS server not yet implemented — see module-level documentation for planned API and implementation notes"
    )
}
