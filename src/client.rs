use std::time::Duration;

use crate::Error;

/// HTTP and WebSocket client for an OpenEPaperLink access point.
///
/// Constructed via [`Client::builder`]. Holds a reusable HTTP client and
/// pre-computed base URLs for the AP.
// All fields are used by the impl Client methods in tags.rs, config.rs,
// system.rs, ws.rs, etc. — split across files via separate impl blocks.
#[allow(dead_code)]
pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) ws_url: String,
}

// Helpers are used by the impl Client methods across tags.rs, config.rs, etc.
#[allow(dead_code)]
impl Client {
    /// Create a new builder for the given AP host (IP address or hostname).
    pub fn builder(host: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            host: host.into(),
            port: None,
            secure: false,
            timeout: None,
            http_client: None,
        }
    }

    /// Build a URL path relative to the AP's base URL.
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Build a URL with query parameters appended manually.
    pub(crate) fn url_with_params(&self, path: &str, params: &[(&str, &str)]) -> String {
        if params.is_empty() {
            return self.url(path);
        }
        let query: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!("{}{}?{}", self.base_url, path, query.join("&"))
    }

    /// Check an AP response body for error indicators.
    ///
    /// The AP returns HTTP 200 for some failures with error text in the body.
    /// This method returns `Ok(())` if the body looks like a success, or
    /// `Err(Error::Api)` if it contains an error message.
    pub(crate) fn check_response_body(&self, body: &str) -> Result<(), Error> {
        let trimmed = body.trim();
        if trimmed.starts_with("Error") || trimmed.starts_with("error") {
            return Err(Error::Api {
                message: trimmed.to_string(),
            });
        }
        Ok(())
    }
}

/// Builder for [`Client`].
pub struct ClientBuilder {
    host: String,
    port: Option<u16>,
    secure: bool,
    timeout: Option<Duration>,
    http_client: Option<reqwest::Client>,
}

impl ClientBuilder {
    /// Set a custom port (default: 80 for HTTP, 443 for HTTPS).
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Use HTTPS/WSS instead of HTTP/WS.
    ///
    /// Requires the `rustls` or `native-tls` feature to be enabled.
    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set a request timeout for all HTTP requests.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Use a pre-built [`reqwest::Client`] instead of constructing one.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Build the [`Client`].
    pub fn build(self) -> Result<Client, Error> {
        let http_scheme = if self.secure { "https" } else { "http" };
        let ws_scheme = if self.secure { "wss" } else { "ws" };
        let default_port = if self.secure { 443 } else { 80 };
        let port = self.port.unwrap_or(default_port);

        let port_suffix = if port == default_port {
            String::new()
        } else {
            format!(":{port}")
        };

        let base_url = format!("{http_scheme}://{}{port_suffix}", self.host);
        let ws_url = format!("{ws_scheme}://{}{port_suffix}/ws", self.host);

        let http = match self.http_client {
            Some(client) => client,
            None => {
                let mut builder = reqwest::Client::builder();
                if let Some(timeout) = self.timeout {
                    builder = builder.timeout(timeout);
                }
                builder.build().map_err(|e| Error::ClientBuild {
                    reason: e.to_string(),
                })?
            }
        };

        Ok(Client {
            http,
            base_url,
            ws_url,
        })
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("ws_url", &self.ws_url)
            .finish()
    }
}
