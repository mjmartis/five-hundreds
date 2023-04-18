// Provides functions that transmit API types to and from WebSocket connections. Allows game code
// to communicate with clients via abstract channels.

use std::debug_assert;

use crate::api;
use crate::events;

use futures_util::SinkExt;
use futures_util::StreamExt;
use nanoid;
use serde_json;
use tokio::sync::mpsc;
use tokio_tungstenite as tokio_ws2;
use tokio_ws2::tungstenite as ws2;

// Connects handles to receive messages from, and to bootstrap outgoing channels to, TCP web clients.
pub fn connect_bridge(addr: String) -> events::EventReceiver {
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

// Spawns two non-blocking threads:
//   1) A thread that transmits JSON payloads from the WebSocket client as steps for the game
//      engine, and
//   2) A thread that transmits states from the game engine into JSON payloads for the client to
//      receive over WebSockets.
//
// Before doing anything else, the former thread transmits a special payload to the engine that is
// used to bootstrap the channel in the latter thread.
fn init_client_socket(
    websocket: tokio_ws2::WebSocketStream<tokio::net::TcpStream>,
    client_id: events::ClientId,
    step_tx: mpsc::UnboundedSender<events::ClientEvent>,
) {
    let (mut write, mut read) = websocket.split();

    // Spawn a thread that transmits messages from the web socket to the game engine. Start with a
    // special message that contains the state sender.
    let (state_tx, mut state_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        // Attempt to send the transmitting end of the state channel. If we can't get replies back,
        // abort immediately.
        let state_tx_payload = events::ClientEvent {
            id: client_id.clone(),
            payload: events::ClientPayload::StateSender(state_tx),
        };
        if step_tx.send(state_tx_payload).is_err() {
            // TODO log error.
            return;
        }

        loop {
            let Some(result) = read.next().await else {
                // The connection has been closed by the client. We send a synthetic "leave" step
                // so that the game engine is aware of the departure.
                let step_payload = events::ClientEvent {
                    id: client_id.clone(),
                    payload: events::ClientPayload::Step(api::Step::Leave),
                };
                step_tx.send(step_payload);
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

            let step_payload = events::ClientEvent {
                id: client_id.clone(),
                payload: events::ClientPayload::Step(step),
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
