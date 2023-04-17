// Provides functions that transmit API types to and from WebSocket connections. Allows game code
// to communicate with clients via abstract channels.

use std::debug_assert;

use crate::api;

use futures_util::SinkExt;
use futures_util::StreamExt;
use nanoid;
use serde_json;
use tokio::sync::mpsc;
use tokio_tungstenite as tokio_ws2;
use tokio_ws2::tungstenite as ws2;

// Unique ID used to identify new and resuming clients from the game engine.
pub type ClientId = String;

// Either a step, or a handle with which the game engine should reply to the client.
pub type StateSender = mpsc::UnboundedSender<api::State>;
pub enum ClientPayload {
    Step(api::Step),
    StateSender(StateSender),
}

// The data sent from a client to the game engine.
pub struct ClientStep {
    pub id: ClientId,
    pub payload: ClientPayload,
}

// An async iterator over steps that a client might take.
pub type Receiver = mpsc::UnboundedReceiver<ClientStep>;

// Connects handles to receive messages from, and to bootstrap outgoing channels to, TCP web clients.
pub fn connect_to_clients(addr: String) -> Receiver {
    debug_assert!(!addr.is_empty());

    let (tx, rx) = mpsc::unbounded_channel();

    // Must always be done on the same thread because we aren't guaranteed inter-thread uniqueness.
    let client_id = nanoid::nanoid!();

    // Spawn here so this function can nicely return the receiver synchronously.
    tokio::spawn(async move {
        // Listen for TCP requests incoming to the given address.
        let listener = tokio::net::TcpListener::bind(addr.clone())
            .await
            .expect("Failed to connect to {addr}");

        loop {
            // TODO: log connection successes and errors.

            // Establish the TCP connection.
            let Ok((stream, _)) = listener.accept().await else { continue };

            // Establish the WebSocket connection.
            let Ok(websocket) = tokio_ws2::accept_async(stream).await else { continue };

            init_client_socket(websocket, client_id.clone(), tx.clone());
        }
    });

    rx
}

// TODO doc
fn init_client_socket(
    websocket: tokio_ws2::WebSocketStream<tokio::net::TcpStream>,
    client_id: String,
    step_tx: mpsc::UnboundedSender<ClientStep>,
) {
    let (mut write, mut read) = websocket.split();

    // Spawn a thread that transmits messages from the web socket to the game engine. Start with a
    // special message that contains the state sender.
    let (state_tx, mut state_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        // Attempt to send the transmitting end of the state channel. If we can't get replies back,
        // abort immediately.
        let state_tx_payload = ClientStep {
            id: client_id.clone(),
            payload: ClientPayload::StateSender(state_tx),
        };
        if step_tx.send(state_tx_payload).is_err() {
            // TODO log error.
            return;
        }

        loop {
            let Some(result) = read.next().await else {
                // The connection has been closed by the client.
                return;
            };

            let Ok(ws2::Message::Text(json)) = result else {
                // TODO log tcp error.
                continue;
            };

            let Ok(step) = serde_json::from_str(&json) else {
                // TODO log malformed error. Transmit error state?
                continue;
            };

            let step_payload = ClientStep {
                id: client_id.clone(),
                payload: ClientPayload::Step(step),
            };
            if step_tx.send(step_payload).is_err() {
                // The connection has been closed by the server.
                return;
            }
        }
    });

    // Spawn a thread that transmits state messages sent from the game engine (through the
    // bootstrapped state pipe) to the web socket.
    tokio::spawn(async move {
        loop {
            let Some(state) = state_rx.recv().await else {
                // The connection has been closed.
                break;
            };

            if write.send(state_to_msg(&state)).await.is_err() {
                // TODO log error.
                return;
            }
        }
    });
}

fn state_to_msg(state: &api::State) -> ws2::Message {
    // We assume our internal data structures can be serialized, and are willing to
    // crash if not.
    ws2::Message::Text(serde_json::to_string(&state).unwrap())
}
