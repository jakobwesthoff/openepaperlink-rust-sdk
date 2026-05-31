/// Errors returned by the OpenEPaperLink SDK.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An HTTP request to the AP failed at the transport level.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The WebSocket connection encountered an error.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// A JSON response could not be deserialized into the expected type.
    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// A MAC address string could not be parsed.
    #[error("invalid MAC address: {reason}")]
    InvalidMac {
        /// What went wrong during parsing.
        reason: String,
    },

    /// The AP accepted the request (HTTP 200) but the response body
    /// indicates a logical error.
    #[error("AP returned error: {message}")]
    Api {
        /// The error message from the AP's response body.
        message: String,
    },

    /// An LED flash pattern string is malformed.
    #[error("invalid LED flash pattern: {reason}")]
    InvalidLedPattern {
        /// What went wrong during parsing.
        reason: String,
    },

    /// The client builder was misconfigured.
    #[error("client build failed: {reason}")]
    ClientBuild {
        /// What was wrong with the builder configuration.
        reason: String,
    },
}
