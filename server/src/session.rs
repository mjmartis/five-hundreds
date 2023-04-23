// The top-level instance of a 500s session. Coordinates the lobby, bidding and gameplay for one
// match.

use std::debug_assert;

use crate::api;
use crate::events;
use crate::events::ClientEventPayload::Connect;
use crate::events::ClientEventPayload::Disconnect;
use crate::events::ClientEventPayload::Step;
use crate::stages;

use log::info;

pub struct Session {
    event_rx: events::ClientEventReceiver,
    clients: events::ClientMap,

    // The client IDs and state histories for each playing player. There can be clients who aren't
    // players, for example when they are unsuccessfully trying to join a full game.
    //
    // TODO: support leaving and rejoining.
    players: Vec<(events::ClientId, api::History)>,

    // The major stage of the session (e.g. lobby, bidding, playing tricks) that we are currently
    // in.
    //
    // We use an Option here so that we can pass ownership into the stage method and then take it
    // back.
    stage: Option<Box<dyn stages::Stage>>,
}

impl Session {
    pub fn new(event_rx: events::ClientEventReceiver) -> Self {
        Self {
            event_rx,
            clients: events::ClientMap::new(),
            players: Vec::new(),
            stage: Some(Box::new(stages::Lobby::new(0))),
        }
    }

    pub async fn run_main_loop(&mut self) {
        loop {
            let Some(event) = self.event_rx.recv().await else {
                info!("All clients dropped - exiting.");
                return;
            };

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

                // Channel to client dropped.
                events::ClientEvent {
                    id,
                    payload: Disconnect,
                } => {
                    self.clients.remove_client(id);

                    if self.player_index(id).is_some() {
                        info!("Player [client {}] disconnected.", id);

                        // Let everyone else know the game can't continue.
                        for (player_id, history) in &self.players {
                            if *player_id == *id {
                                continue;
                            }

                            self.clients.send_event(
                                player_id,
                                Some(history.clone()),
                                api::CurrentState::MatchAborted("Player disconnected".to_string()),
                            );
                        }

                        self.stage = Some(Box::new(stages::Aborted {}));
                        continue;
                    }
                }

                // A client has left. This might end the game if they are an active player. We
                // handle this here because a player can quit from any stage.
                events::ClientEvent {
                    id,
                    payload: Step(api::Step::Quit),
                } => {
                    // Active player has left.
                    if self.player_index(id).is_some() {
                        // Let everyone know the game can't continue.
                        for (player_id, history) in &self.players {
                            self.clients.send_event(
                                player_id,
                                Some(history.clone()),
                                api::CurrentState::MatchAborted("Player left".to_string()),
                            );
                        }

                        info!("Player [client {}] left.", id);
                        self.stage = Some(Box::new(stages::Aborted {}));
                    } else {
                        info!("[client {}] tried to leave without joining.", id);
                        self.clients.send_event(
                            id,
                            None,
                            api::CurrentState::Error("Tried to leave without joining.".to_string()),
                        );
                    }
                }

                // A step sent from a client. Delegate handling to individual stage
                // implementations.
                events::ClientEvent {
                    id,
                    payload: Step(step),
                } => {
                    let player_index = self.player_index(id);

                    // Give up and then retake ownership of the stage object.
                    let stage = self.stage.take();
                    debug_assert!(stage.is_some());
                    self.stage = Some(stage.unwrap().process_step(
                        &mut self.players,
                        player_index,
                        &self.clients,
                        id,
                        step,
                    ));
                }
            };
        }
    }

    // Returns the index in the player list of the given client ID, if it is present.
    fn player_index(&self, id: &events::ClientId) -> Option<usize> {
        self.players.iter().position(|(i, _)| i == id)
    }
}
