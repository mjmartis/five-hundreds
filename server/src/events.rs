// Types used to communicate between clients and servers. Stored in their own module to separate
// them conceptually from the "web bridge" that ferries them from web clients.

use crate::api;

use tokio::sync::mpsc;

// Unique ID used to identify new and resuming clients from the game engine.
pub type ClientId = u128;

// The events that could originate from the client. Notably, one option is a handle with which the
// game engine should subsequently reply to the client.
pub enum ClientPayload {
    Step(api::Step),
    StateSender(StateSender),
    Disconnect,
}

// The data sent from a client to the game engine.
pub struct ClientEvent {
    pub id: ClientId,
    pub payload: ClientPayload,
}

// An async iterator over messages that a client might send.
pub type EventReceiver = mpsc::UnboundedReceiver<ClientEvent>;

// An async transmitter used to send states to a client.
pub type StateSender = mpsc::UnboundedSender<api::State>;
