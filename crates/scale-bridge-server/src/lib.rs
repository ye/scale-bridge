//! HTTPS REST server for scale-bridge.
//!
//! Exposes scale readings over HTTP/JSON so web applications can query
//! scales without a direct serial connection.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use scale_bridge_core::{EtxCodec, Scale, ScaleError, SerialTransport, TcpTransport, Transport};
#[cfg(test)]
use scale_bridge_core::MockTransport;
use scale_bridge_scp01::{
    AboutInfo, DiagnosticInfo, MetrologyReading, NciCommand, NciProtocol, NciResponse,
    ScaleStatus, WeightReading,
};
use serde::Serialize;
use std::{
    io::{Read, Write},
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

/// Server configuration.
pub struct ServerConfig {
    pub https_port: u16,
    pub bind_addr: String,
    pub scale_serial_port: Option<String>,
    pub scale_host: Option<String>,
    pub scale_tcp_port: u16,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

enum ServerTransport {
    Serial(SerialTransport),
    Tcp(TcpTransport),
    #[cfg(test)]
    Mock(MockTransport),
}

impl Read for ServerTransport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            ServerTransport::Serial(t) => t.read(buf),
            ServerTransport::Tcp(t) => t.read(buf),
            #[cfg(test)]
            ServerTransport::Mock(t) => t.read(buf),
        }
    }
}

impl Write for ServerTransport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            ServerTransport::Serial(t) => t.write(buf),
            ServerTransport::Tcp(t) => t.write(buf),
            #[cfg(test)]
            ServerTransport::Mock(t) => t.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            ServerTransport::Serial(t) => t.flush(),
            ServerTransport::Tcp(t) => t.flush(),
            #[cfg(test)]
            ServerTransport::Mock(t) => t.flush(),
        }
    }
}

impl Transport for ServerTransport {
    fn set_timeout(&mut self, timeout: std::time::Duration) -> Result<(), ScaleError> {
        match self {
            ServerTransport::Serial(t) => t.set_timeout(timeout),
            ServerTransport::Tcp(t) => t.set_timeout(timeout),
            #[cfg(test)]
            ServerTransport::Mock(t) => t.set_timeout(timeout),
        }
    }

    fn flush_output(&mut self) -> Result<(), ScaleError> {
        match self {
            ServerTransport::Serial(t) => t.flush_output(),
            ServerTransport::Tcp(t) => t.flush_output(),
            #[cfg(test)]
            ServerTransport::Mock(t) => t.flush_output(),
        }
    }
}

type SharedScale = Arc<Mutex<Scale<ServerTransport, EtxCodec, NciProtocol>>>;

#[derive(Clone)]
struct AppState {
    scale: SharedScale,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<serde_json::Value>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}

impl ApiError {
    fn new(status: StatusCode, error: impl Into<String>) -> Self {
        Self {
            status,
            body: ErrorBody {
                error: error.into(),
                detail: None,
            },
        }
    }

    fn with_detail(
        status: StatusCode,
        error: impl Into<String>,
        detail: impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            status,
            body: ErrorBody {
                error: error.into(),
                detail: Some(serde_json::to_value(detail)?),
            },
        })
    }
}

impl From<ScaleError> for ApiError {
    fn from(err: ScaleError) -> Self {
        match err {
            ScaleError::UnrecognizedCommand => {
                ApiError::new(StatusCode::NOT_IMPLEMENTED, err.to_string())
            }
            ScaleError::Timeout => ApiError::new(StatusCode::GATEWAY_TIMEOUT, err.to_string()),
            ScaleError::Transport(_) | ScaleError::SerialPort(_) => {
                ApiError::new(StatusCode::BAD_GATEWAY, err.to_string())
            }
            ScaleError::FramingError(_) | ScaleError::ParseError(_) => {
                ApiError::new(StatusCode::BAD_GATEWAY, err.to_string())
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

async fn send_command(state: AppState, cmd: NciCommand) -> Result<NciResponse, ApiError> {
    tokio::task::spawn_blocking(move || {
        let mut scale = state
            .scale
            .lock()
            .map_err(|_| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "scale lock poisoned"))?;
        scale.send(cmd).map_err(ApiError::from)
    })
    .await
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
}

async fn weight(State(state): State<AppState>) -> Result<Json<WeightReading>, ApiError> {
    match send_command(state, NciCommand::Weight).await? {
        NciResponse::Weight(w) | NciResponse::HighResolution(w) => Ok(Json(w)),
        NciResponse::Status(s) => Err(
            ApiError::with_detail(
                StatusCode::CONFLICT,
                "scale returned status instead of weight",
                s,
            )
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        ),
        other => Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            format!("unexpected response for weight: {other:?}"),
        )),
    }
}

async fn status(State(state): State<AppState>) -> Result<Json<ScaleStatus>, ApiError> {
    match send_command(state, NciCommand::Status).await? {
        NciResponse::Status(s) => Ok(Json(s)),
        other => Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            format!("unexpected response for status: {other:?}"),
        )),
    }
}

async fn metrology(State(state): State<AppState>) -> Result<Json<MetrologyReading>, ApiError> {
    match send_command(state, NciCommand::Metrology).await? {
        NciResponse::Metrology(m) => Ok(Json(m)),
        other => Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            format!("unexpected response for metrology: {other:?}"),
        )),
    }
}

async fn about(State(state): State<AppState>) -> Result<Json<AboutInfo>, ApiError> {
    match send_command(state, NciCommand::About).await? {
        NciResponse::About(a) => Ok(Json(a)),
        other => Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            format!("unexpected response for about: {other:?}"),
        )),
    }
}

async fn diagnostic(State(state): State<AppState>) -> Result<Json<DiagnosticInfo>, ApiError> {
    match send_command(state, NciCommand::Diagnostic).await? {
        NciResponse::Diagnostic(d) => Ok(Json(d)),
        other => Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            format!("unexpected response for diagnostic: {other:?}"),
        )),
    }
}

async fn zero(State(state): State<AppState>) -> Result<StatusCode, ApiError> {
    match send_command(state, NciCommand::Zero).await? {
        NciResponse::Acknowledged | NciResponse::Status(_) => Ok(StatusCode::NO_CONTENT),
        other => Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            format!("unexpected response for zero: {other:?}"),
        )),
    }
}

async fn tare(State(state): State<AppState>) -> Result<StatusCode, ApiError> {
    match send_command(state, NciCommand::Tare).await? {
        NciResponse::Acknowledged | NciResponse::Status(_) => Ok(StatusCode::NO_CONTENT),
        other => Err(ApiError::new(
            StatusCode::BAD_GATEWAY,
            format!("unexpected response for tare: {other:?}"),
        )),
    }
}

async fn health() -> StatusCode {
    StatusCode::OK
}

fn build_transport(config: &ServerConfig) -> Result<ServerTransport, ScaleError> {
    if let Some(port) = &config.scale_serial_port {
        return Ok(ServerTransport::Serial(SerialTransport::open(
            port,
            9600,
            serialport::Parity::Even,
        )?));
    }

    if let Some(host) = &config.scale_host {
        return Ok(ServerTransport::Tcp(TcpTransport::connect(
            host,
            config.scale_tcp_port,
        )?));
    }

    Err(ScaleError::Transport(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "specify scale_serial_port or scale_host",
    )))
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/weight", get(weight))
        .route("/api/status", get(status))
        .route("/api/metrology", get(metrology))
        .route("/api/about", get(about))
        .route("/api/diagnostic", get(diagnostic))
        .route("/api/zero", post(zero))
        .route("/api/tare", post(tare))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .with_state(state)
}

/// Start the HTTPS server.
pub fn serve(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move {
        let cert_path = config
            .cert_path
            .clone()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing --cert"))?;
        let key_path = config
            .key_path
            .clone()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing --key"))?;

        let tls = RustlsConfig::from_pem_file(cert_path, key_path).await?;
        let transport = build_transport(&config)?;
        let state = AppState {
            scale: Arc::new(Mutex::new(Scale::new(transport, EtxCodec::new(), NciProtocol))),
        };
        let app = app(state);
        let addr = SocketAddr::from_str(&format!("{}:{}", config.bind_addr, config.https_port))?;

        tracing::info!("scale-bridge-server listening on https://{addr}");

        axum_server::bind_rustls(addr, tls)
            .serve(app.into_make_service())
            .await?;

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use scale_bridge_core::MockTransport;
    use tower::util::ServiceExt;

    fn app_with_response(response: Vec<u8>) -> Router {
        let transport = ServerTransport::Mock(MockTransport::with_response(response));
        let state = AppState {
            scale: Arc::new(Mutex::new(Scale::new(transport, EtxCodec::new(), NciProtocol))),
        };
        app(state)
    }

    #[tokio::test]
    async fn healthz_returns_ok() {
        let app = app_with_response(vec![]);
        let response = app
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn weight_returns_json_payload() {
        let response_bytes = b"\x0a  1234.56lb\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        let app = app_with_response(response_bytes);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/weight")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["value"], "1234.56");
        assert_eq!(value["unit"], "lb");
    }

    #[tokio::test]
    async fn status_returns_json_payload() {
        let app = app_with_response(b"\x0aS00\x0d\x03".to_vec());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["motion"], false);
        assert_eq!(value["at_zero"], false);
    }

    #[tokio::test]
    async fn weight_returns_conflict_when_scale_replies_with_status_only() {
        let app = app_with_response(b"\x0aS01\x0d\x03".to_vec());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/weight")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["error"], "scale returned status instead of weight");
        assert_eq!(value["detail"]["at_zero"], true);
    }

    #[tokio::test]
    async fn zero_returns_no_content_for_status_reply() {
        let app = app_with_response(b"\x0aS00\x0d\x03".to_vec());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/zero")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(body.is_empty());
    }
}
