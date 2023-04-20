// Types used to communicate between clients and servers. Stored in their own module to separate
// them conceptually from the "web bridge" that ferries them from web clients.

use crate::api;

use tokio::sync::mpsc;

// Unique ID used to identify new and resuming clients from the game engine.
pub type ClientId = u128;

// The events that could originate from the client. Notably, one option is a handle with which the
// game engine should subsequently reply to the client.
pub enum ClientEventPayload {
    Step(api::Step),
    EngineEventSender(EngineEventSender),
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
