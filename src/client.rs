use std::time::Duration;

use crate::Error;

/// HTTP and WebSocket client for an OpenEPaperLink access point.
///
/// Constructed via [`Client::builder`]. Holds a reusable HTTP client and
/// pre-computed base URLs for the AP.
pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) ws_url: String,
}

impl Client {
    /// Create a new builder for the given AP base URL (e.g. `http://192.168.1.100`).
    pub fn builder(base_url: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            base_url: base_url.into(),
            port: None,
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
        if trimmed.to_ascii_lowercase().starts_with("error") {
            return Err(Error::Api {
                message: trimmed.to_string(),
            });
        }
        Ok(())
    }
}

/// Builder for [`Client`].
pub struct ClientBuilder {
    base_url: String,
    port: Option<u16>,
    timeout: Option<Duration>,
    http_client: Option<reqwest::Client>,
}

impl ClientBuilder {
    /// Override the port in the base URL.
    ///
    /// Replaces any existing port, or inserts one if none was present.
    ///
    /// TODO: IPv6 bracket addresses (e.g. `http://[::1]:8080`) are not
    /// handled by the port-override logic and will produce a malformed URL.
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
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
        // Strip a single trailing slash so callers who write
        // "http://192.168.1.100/" and those who don't get the same result.
        let raw = self.base_url.strip_suffix('/').unwrap_or(&self.base_url);

        // Apply a port override by rewriting the authority portion of the URL.
        // We split on "://" to isolate the scheme, then further split the
        // authority (host[:port]) from any trailing path.
        let base_url = if let Some(port) = self.port {
            let (scheme, rest) = raw.split_once("://").ok_or_else(|| Error::ClientBuild {
                reason: format!("base URL has no scheme: {raw}"),
            })?;
            // Split authority from any path that follows it.
            let (authority, path) = rest
                .split_once('/')
                .map(|(a, p)| (a, format!("/{p}")))
                .unwrap_or((rest, String::new()));
            // Drop any existing port from the authority.
            let host = authority.split_once(':').map_or(authority, |(h, _)| h);
            format!("{scheme}://{host}:{port}{path}")
        } else {
            raw.to_string()
        };

        // Derive the WebSocket URL by swapping the scheme. We check for
        // "https://" before "http://" to avoid a prefix collision.
        let ws_url = if let Some(rest) = base_url.strip_prefix("https://") {
            format!("wss://{rest}/ws")
        } else if let Some(rest) = base_url.strip_prefix("http://") {
            format!("ws://{rest}/ws")
        } else {
            return Err(Error::ClientBuild {
                reason: format!("base URL must start with http:// or https://: {base_url}"),
            });
        };

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
