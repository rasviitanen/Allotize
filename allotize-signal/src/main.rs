mod auth;

use dashmap::DashMap;
use eyre::{eyre, Result};
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, info, instrument};
use warp::http::StatusCode;
use warp::ws::{Message, WebSocket};
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

#[derive(Serialize, Debug)]
struct ErrorResponse {
    message: String,
    status: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum Protocol {
    OneToAll,
    OneToOne,
    OneToRoom,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum SignalingAction {
    Answer,
    Candidate,
    HandleConnection,
    Heartbeat,
    Offer,
    ReconnectOffer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SignalingMessage {
    action: SignalingAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: Option<String>,
    from: String,
    protocol: Protocol,
    room: String,
}

#[derive(Debug)]
struct Node {
    sender: mpsc::UnboundedSender<Result<Message, warp::Error>>,
}

#[derive(Debug, Clone, Default)]
struct SignalingServer {
    rooms: Arc<DashMap<String, HashMap<String, Node>>>,
}

impl SignalingServer {
    fn new() -> Self {
        SignalingServer {
            rooms: Default::default(),
        }
    }

    #[instrument(skip(self, ws))]
    async fn connect_user(&self, room: String, username: String, ws: WebSocket) {
        let (user_ws_tx, mut user_ws_rx) = ws.split();
        let (sender, rx) = mpsc::unbounded_channel();
        let rx = UnboundedReceiverStream::new(rx);

        tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
            if let Err(e) = result {
                error!("websocket send error: {}", e);
            }
        }));
        info!("connected");

        self.rooms
            .entry(room.clone())
            .or_default()
            .insert(username.clone(), Node { sender });

        while let Some(msg) = user_ws_rx.next().await {
            let msg: SignalingMessage = match (|| {
                Ok::<SignalingMessage, eyre::Report>(serde_json::from_str(
                    msg?.to_str()
                        .map_err(|_| eyre!("failed to convert to stirng"))?,
                )?)
            })() {
                Ok(msg) => msg,
                Err(e) => {
                    error!(%e, %username);
                    continue;
                }
            };

            if let Some(room) = self.rooms.get(&room) {
                match &msg.protocol {
                    Protocol::OneToOne => {
                        if let Some(endpoint) = msg
                            .endpoint
                            .as_ref()
                            .and_then(|endpoint| room.get(endpoint))
                        {
                            let _ = endpoint
                                .sender
                                .send(Ok(Message::text(serde_json::to_string(&msg).unwrap())));
                        }
                    }
                    _ => {
                        for (node_name, node) in room.iter() {
                            if *node_name == username {
                                continue;
                            }
                            info!("sending to node in room: {:?}", &msg);
                            let _ = node
                                .sender
                                .send(Ok(Message::text(serde_json::to_string(&msg).unwrap())));
                        }
                    }
                }
            }
        }

        if let Some(mut room) = self.rooms.get_mut(&room) {
            room.remove(&username);
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthResponse {
    session_token: String,
}

#[instrument]
async fn auth(x_api_key: String) -> Result<impl warp::Reply, warp::Rejection> {
    if !auth::KEYS.contains(&x_api_key.as_str()) {
        return Err(warp::reject::custom(WebError::InvalidAPIKey));
    }

    let response = AuthResponse {
        session_token: "ok".to_string(),
    };

    let token = auth::Claims::new(
        x_api_key,
        "allotize.com".to_string(),
        chrono::Duration::days(1),
    )
    .tokenize();

    let reply = warp::reply::json(&response);

    if let Ok(token) = token {
        Ok(warp::reply::with_header(reply, "Authorization", &token))
    } else {
        Err(warp::reject::custom(WebError::InvalidAPIKey))
    }
}

#[derive(Error, Debug)]
pub enum WebError {
    #[error("invalid API key")]
    InvalidAPIKey,
}

impl warp::reject::Reject for WebError {}

fn routes() -> BoxedFilter<(impl warp::Reply,)> {
    let signaling_server = SignalingServer::new();
    let signaling_server = warp::any().map(move || signaling_server.clone());

    let routes = warp::path("hello").map(|| "hello!");

    let routes = routes.or(warp::path("auth")
        .and(warp::post())
        .and(warp::header("X-API-Key"))
        .and_then(auth));

    let routes = routes.or(warp::path("connect")
        .and(warp::path::param())
        .and(warp::path::param())
        .and(warp::header("X-API-Key"))
        .and(warp::ws())
        .and(signaling_server)
        .and_then(
            |room: String,
             username: String,
             x_api_key: String,
             ws: warp::ws::Ws,
             signaling_server: SignalingServer| async move {
                if !auth::KEYS.contains(&x_api_key.as_str()) {
                    return Err(warp::reject::custom(WebError::InvalidAPIKey));
                }

                Ok(ws.on_upgrade(move |socket| async move {
                    signaling_server.connect_user(room, username, socket).await;
                }))
            },
        ));

    routes
        .recover(handle_rejection)
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_header("X-API-KEY")
                .allow_methods(vec!["GET", "POST"]),
        )
        .boxed()
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<WebError>() {
        match e {
            WebError::InvalidAPIKey => (StatusCode::UNAUTHORIZED, e.to_string()),
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed".to_string(),
        )
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    let json = warp::reply::json(&ErrorResponse {
        status: code.to_string(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let routes = routes();
    let index = warp::path::end().map(|| warp::reply::html(TEST_HTML));

    let port = std::env::var("PORT")
        .map(|port| port.parse::<u16>().unwrap_or(3030))
        .unwrap_or(3030);
    warp::serve(index.or(routes))
        .run(([0, 0, 0, 0], port))
        .await;
}

static TEST_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Allotize Signal</title>
    </head>
    <body>
        <h1>Allotize Signal Example</h1>
        <script type="text/javascript">
        const uri = 'ws://' + location.host + '/connect/room1/user1';
        const ws = new WebSocket(uri);

        ws.onopen = function() {
            console.log("Socket Opened");
            ws.send("hello");
        };
        ws.onmessage = function(msg) {
            console.log("Socket Message");
        };
        ws.onclose = function() {
            console.log("Socket Closed");
        };
        </script>
    </body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth() {
        let filter = routes();

        let res = warp::test::request().path("/auth").reply(&filter).await;
        assert_eq!(res.status(), 405, "GET is not allowed");

        let res = warp::test::request()
            .method("POST")
            .path("/auth")
            .header("X-API-KEY", "ZXlKMGVYQWlPaUpLVjFRaUxDSmhiR2NpT2lKSVV6STFOaUo5LmV5SnpkV0lpT2lKeVlYTjJhU0lzSW1GMVpDSTZJbWgwZEhCek9pOHZZV3hzYjNScGVtVXVZMjl0SWl3aVpYaHdJam94TmpVM01Ea3hNVGN4ZlEuR09YQnRRTGJYUHRtYkhDSy00b3pMSnI1Q09QRzgtMXNzTjgtMWROeXlmQQ==")
            .reply(&filter)
            .await;
        assert_eq!(res.status(), 200);
        dbg!(res.headers().get("Authorization"));
    }
}
