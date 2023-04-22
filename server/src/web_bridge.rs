// Provides functions that transmit API types to and from WebSocket connections. Allows game code
// to communicate with clients via abstract channels.

use std::debug_assert;

use crate::api;
use crate::events;

use futures_util::SinkExt;
use futures_util::StreamExt;
use log::{debug, error, info};
use serde_json;
use tokio::sync::mpsc;
use tokio_tungstenite as tokio_ws2;
use tokio_ws2::tungstenite as ws2;
use unique_id;
use unique_id::random::RandomGenerator;
use unique_id::Generator;

// Connects a handle to receive messages from TCP web clients. One type of message is a
// "transmitter" message that lets the handler send messages back to clients.
pub fn connect_bridge(addr: String) -> events::ClientEventReceiver {
    debug_assert!(!addr.is_empty());

    let (tx, rx) = mpsc::unbounded_channel();

    // Spawn here so this function can nicely return the receiver synchronously.
    tokio::spawn(async move {
        // Listen for TCP requests incoming to the given address.
        let listener = tokio::net::TcpListener::bind(addr.clone())
            .await
            .expect("Failed to connect to {addr}.");

        loop {
            // Establish the TCP connection.
            let Ok((stream, client_addr)) = listener.accept().await else {
                error!("Couldn't connect to TCP stream.");
                continue;
            };

            // Guaranteed to be unique amongst all threads.
            let client_id = RandomGenerator::default().next_id();
            info!(
                "[client {}] connected to TCP stream at {}.",
                client_id, &client_addr
            );

            // Establish the WebSocket connection.
            let Ok(websocket) = tokio_ws2::accept_async(stream).await else {
                error!("[client {}] couldn't establish WebSocket connection with {}.", client_id, &client_addr);
                continue;
            };
            info!(
                "[client {}] established WebSocket connection with {}.",
                client_id, &client_addr
            );

            init_client_socket(websocket, client_id, tx.clone());
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
// Before doing anything else, the former thread transmits a special "transmitter" payload that the
// engine can use to send its states to the latter thread.
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
            id: client_id,
            payload: events::ClientEventPayload::Connect(state_tx),
        };
        if step_tx.send(state_tx_payload).is_err() {
            error!("Couldn't send reply channel for [client {}].", client_id);
            return;
        }

        loop {
            let Some(result) = read.next().await else {
                info!("WebSocket connection closed by [client {}].", client_id);

                let disconnect_payload = events::ClientEvent {
                    id: client_id,
                    payload: events::ClientEventPayload::Disconnect,
                };
                if step_tx.send(disconnect_payload).is_err() {
                    debug!("Channel to [client {}] closed by the engine.", client_id);
                }

                return;
            };

            let Ok(ws2::Message::Text(json)) = result else {
                error!("Malformed message sent from [client {}].", client_id);
                continue;
            };

            let Ok(step) = serde_json::from_str(&json) else {
                error!("Malformed JSON sent from [client {}].", client_id);
                continue;
            };

            let step_payload = events::ClientEvent {
                id: client_id,
                payload: events::ClientEventPayload::Step(step),
            };
            if step_tx.send(step_payload).is_err() {
                debug!("Channel to [client {}] closed by the engine.", client_id);
                return;
            }
        }
    });

    // Spawn a thread that transmits state messages sent from the game engine to the web socket.
    // The engine can transmit these messages after it has been passed a handle through the initial
    // "transmitter" message.
    tokio::spawn(async move {
        loop {
            let Some(state) = state_rx.recv().await else {
                debug!("Channel to [client {}] closed by the engine.", client_id);
                return;
            };

            // We assume our internal data structures can be serialized, and are willing to
            // crash if not.
            let msg = ws2::Message::Text(serde_json::to_string(&state).unwrap());
            if write.send(msg).await.is_err() {
                error!(
                    "Failed to send message to WebSocket for [client {}].",
                    client_id
                );
                return;
            }
        }
    });
}
