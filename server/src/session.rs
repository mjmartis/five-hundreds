// The top-level instance of a 500s session. Coordinates the lobby, bidding and gameplay for one
// match.

use std::collections::HashMap;
use log::{info, error};

use crate::api;
use crate::events;
use crate::types;
use crate::events::ClientPayload::StateSender;
use crate::events::ClientPayload::Step;


pub struct Session {
    event_rx: events::EventReceiver,
    client_txs: HashMap<events::ClientId, events::StateSender>,

    // TODO: break this data out into objects that can be shared.

    state: InternalState,

    // The client IDs of playing players. There can be clients who aren't players, for example when
    // they are unsuccessfully trying to join a full game.
    //
    // TODO: support leaving and rejoining.
    players: Vec<String>,
}

impl Session {
    pub fn new(event_rx: events::EventReceiver) -> Self {
        Self { event_rx, client_txs: HashMap::new(), state: InternalState::Lobby, players: Vec::new() }
    }

    pub async fn run_main_loop(self: &mut Self) {
        loop {
            let Some(event) = self.event_rx.recv().await else {
                info!("All clients dropped - exiting.");
                return;
            };

            // First, handle connection-related events.
            match &event {
                // New response channel received.
                events::ClientEvent {
                    id,
                    payload: StateSender(tx),
                } => {
                    self.client_txs.insert(id.clone(), tx.clone());
                    info!("New [client {}] registered with engine.", id);
                    continue;
                },

                // A client has left. This might end the game if they are an active player.
                // player.
                events::ClientEvent {
                    id,
                    payload: Step(api::Step::Leave),
                } => {
                    info!("[client {}] left.", id);
                    self.client_txs.remove(id);
                    if self.players.contains(id) {
                        // TODO: send all clients goodbye messages.
                        return;
                    }
                },

                _ => {},
            };

            // Otherwise, delegate logic to specialised handlers.
            // TODO: break this logic out into separate objects.
            match &self.state {
                InternalState::Lobby => self.process_lobby_step(event),
                _ => continue,
            }

        };
    }

    fn process_lobby_step(self: &mut Self, event: events::ClientEvent) {
        match event {
            // A client is attempting to join.
            events::ClientEvent {
                id,
                // TODO: respect team request.
                payload: Step(api::Step::Join(_)),
            } => {
                if self.players.contains(&id) {
                    self.send_state(&id, api::State::Excluded("Game ongoing".to_string()));
                    info!("[client {}] excluded because they have already joined.", &id);
                    return;
                }

                if self.players.len() == 4 {
                    self.send_state(&id, api::State::Excluded("Game ongoing".to_string()));
                    info!("[client {}] excluded due to ongoing game.", &id);
                    return;
                }

                self.players.push(id.clone());
                info!("[client {}] joined.", &id);

                // All players newly joined.
                if self.players.len() == 4 {
                    info!("Starting match.");
                    self.state = InternalState::Bidding;

                    // Tell clients that hands have been dealt.
                    // TODO: stop lying to them.
                    for out_id in &self.players {
                        self.send_state(out_id, api::State::HandDealt);
                    }
                }
            },

            _ => {},
        };
    }

    fn send_state(self: &Self, id: &events::ClientId, state: api::State) {
        let Some(tx) = self.client_txs.get(id) else {
            // This is violating an invariant we should have maintained.
            error!("Attempted to send message to unregistered [client {}].", id);
            return;
        };

        if tx.send(state).is_err() {
            error!("Engine couldn't send event to [client {}].", id);
        }
    }
}

enum InternalState {
    Lobby,
    Bidding,
    BidWon,
    Game,
}
