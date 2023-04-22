// The top-level instance of a 500s session. Coordinates the lobby, bidding and gameplay for one
// match.

use crate::api;
use crate::events;
use crate::events::ClientEventPayload::Connect;
use crate::events::ClientEventPayload::Disconnect;
use crate::events::ClientEventPayload::Step;
use crate::types;

use log::{error, info};

pub struct Session {
    event_rx: events::ClientEventReceiver,
    clients: events::ClientMap,

    // TODO: break this data out into objects that can be shared.
    state: InternalState,

    // The client IDs of each playing player. There can be clients who aren't players, for example
    // when they are unsuccessfully trying to join a full game.
    //
    // TODO: support leaving and rejoining.
    players: Vec<events::ClientId>,

    // The histories for each playing player. This vector is always equal in size to the players
    // vector.
    histories: Vec<api::History>,
}

impl Session {
    pub fn new(event_rx: events::ClientEventReceiver) -> Self {
        Self {
            event_rx,
            clients: events::ClientMap::new(),
            state: InternalState::Lobby,
            players: Vec::new(),
            histories: Vec::new(),
        }
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
                    payload: Connect(tx),
                } => {
                    self.clients.add_client(id, tx.clone());
                    info!("New [client {}] connected to engine.", id);
                    continue;
                }

                events::ClientEvent {
                    id,
                    payload: Disconnect,
                } => {
                    self.clients.remove_client(id);
                    if self.players.contains(id) {
                        // TODO: send all clients goodbye messages.
                        info!("Player [client {}] disconnected.", id);
                        self.state = InternalState::MatchAborted;
                        continue;
                    }
                }

                _ => {}
            };

            // Otherwise, delegate logic to specialised handlers.
            // TODO: break this logic out into separate objects.
            match &self.state {
                InternalState::Lobby => self.process_lobby_step(event),
                InternalState::MatchAborted => {
                    self.clients.send_event(
                        &event.id,
                        self.player_history(&event.id),
                        api::CurrentState::MatchAborted("Player left.".to_string()),
                    );
                }
                _ => {}
            }
        }
    }

    fn process_lobby_step(self: &mut Self, event: events::ClientEvent) {
        match &event {
            // A client is attempting to join.
            events::ClientEvent {
                id,
                // TODO: respect team request.
                payload: Step(api::Step::Join(_)),
            } => {
                if self.players.contains(id) {
                    self.clients.send_event(
                        id,
                        self.player_history(id),
                        api::CurrentState::Excluded("Already joined.".to_string()),
                    );
                    info!("[client {}] excluded because they have already joined.", id);
                    return;
                }

                if self.players.len() == 4 {
                    self.clients.send_event(
                        id,
                        self.player_history(id),
                        api::CurrentState::Excluded("Game ongoing.".to_string()),
                    );
                    info!("[client {}] excluded due to ongoing game.", id);
                    return;
                }

                self.players.push(*id);
                self.histories.push(Default::default());
                info!("[client {}] joined.", id);

                // All players newly joined.
                if self.players.len() == 4 {
                    info!("Starting match.");
                    self.state = InternalState::Bidding;

                    // Tell clients that hands have been dealt.
                    // TODO: stop lying to them.
                    for i in 0..self.players.len() {
                        self.clients.send_event(
                            &self.players[i],
                            Some(self.histories[i].clone()),
                            api::CurrentState::HandDealt,
                        );
                    }
                }
            }

            // A client has left. This might end the game if they are an active player.
            // player.
            events::ClientEvent {
                id,
                payload: Step(api::Step::Quit),
            } => {
                if self.players.contains(id) {
                    // TODO: send all clients goodbye messages.
                    info!("Player [client {}] left.", id);
                    self.state = InternalState::MatchAborted;
                } else {
                    info!("[client {}] tried to leave without joining.", id);
                    self.clients.send_event(
                        id,
                        self.player_history(id),
                        api::CurrentState::Error("Tried to leave without joining.".to_string()),
                    );
                }
            }

            _ => {}
        };
    }

    fn player_history(self: &Self, id: &events::ClientId) -> Option<api::History> {
        for i in 0..self.players.len() {
            if self.players[i] == *id {
                return Some(self.histories[i].clone());
            }
        }

        None
    }
}

enum InternalState {
    Lobby,
    Bidding,
    BidWon,
    Game,
    MatchAborted,
}
