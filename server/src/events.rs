// Types used to communicate between clients and servers. Stored in their own module to separate
// them conceptually from the "web bridge" that ferries them from web clients.

use crate::api;

use std::collections::HashMap;

use log::error;
use tokio::sync::mpsc;

// Unique ID used to identify new and resuming clients from the game engine.
pub type ClientId = u128;

// The events that could originate from the client. Notably, one option is a handle with which the
// game engine should subsequently reply to the client.
pub enum ClientEventPayload {
    Step(api::Step),
    Connect(EngineEventSender),
    Disconnect,
}

// The data sent from a client to the game engine.
pub struct ClientEvent {
    pub id: ClientId,
    pub payload: ClientEventPayload,
}

// An async iterator over messages that a client might send.
pub type ClientEventReceiver = mpsc::UnboundedReceiver<ClientEvent>;

// An async transmitter used to send events to a client.
pub type EngineEventSender = mpsc::UnboundedSender<api::State>;

// Used to transmit engine events to a set of clients.
pub struct ClientMap {
    client_txs: HashMap<ClientId, EngineEventSender>,
}

impl ClientMap {
    pub fn new() -> Self {
        Self {
            client_txs: HashMap::new(),
        }
    }

    pub fn add_client(self: &mut Self, id: &ClientId, tx: EngineEventSender) {
        self.client_txs.insert(*id, tx.clone());
    }

    pub fn remove_client(self: &mut Self, id: &ClientId) {
        self.client_txs.remove(id);
    }

    pub fn send_event(
        self: &Self,
        id: &ClientId,
        history: Option<api::History>,
        state: api::CurrentState,
    ) {
        let Some(tx) = self.client_txs.get(id) else {
            error!("Attempted to send message to unregistered [client {}].", id);
            return;
        };

        if tx.send(api::State { state, history }).is_err() {
            error!("Engine couldn't send event to [client {}].", id);
        }
    }
}
