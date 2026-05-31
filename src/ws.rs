use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Stream, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::client::Client;
use crate::{Error, WsMessage};

/// A stream of typed WebSocket messages from the AP.
///
/// Yields `Result<WsMessage, Error>` items. JSON parse errors produce `Err`
/// items without ending the stream. A clean close or connection drop ends
/// the stream. Callers reconnect by calling [`Client::connect_ws`] again.
pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = Result<WsMessage, Error>> + Send>>,
}

impl Stream for EventStream {
    type Item = Result<WsMessage, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

impl Client {
    /// Connect to the AP's WebSocket endpoint and return a typed event stream.
    ///
    /// The stream yields [`WsMessage`] variants for each received message.
    /// On disconnect, the stream ends — call this method again to reconnect.
    pub async fn connect_ws(&self) -> Result<EventStream, Error> {
        let (ws_stream, _response) =
            tokio_tungstenite::connect_async(&self.ws_url).await?;

        let stream = ws_stream.filter_map(|result| async {
            match result {
                Ok(Message::Text(text)) => {
                    Some(serde_json::from_str::<WsMessage>(&text).map_err(Error::from))
                }
                Ok(Message::Close(_)) => None,
                // Binary, ping, pong frames are not used by the AP
                Ok(_) => None,
                Err(e) => Some(Err(Error::from(e))),
            }
        });

        Ok(EventStream {
            inner: Box::pin(stream),
        })
    }
}
