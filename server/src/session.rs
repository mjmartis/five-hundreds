// The top-level instance of a 500s session. Coordinates the lobby, bidding and gameplay for one
// match.

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
            stage: Some(Box::new(stages::Lobby {})),
        }
    }

    pub async fn run_main_loop(self: &mut Self) {
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
                    if let Some(_) = self.player_index(id) {
                        // TODO: send all clients goodbye messages.
                        info!("Player [client {}] disconnected.", id);
                        self.stage = Some(Box::new(stages::Aborted {}));
                        continue;
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

    fn player_index(self: &Self, id: &events::ClientId) -> Option<usize> {
        self.players.iter().position(|(i, _)| i == id)
    }

    fn player_history(self: &Self, id: &events::ClientId) -> Option<api::History> {
        if let Some(i) = self.player_index(id) {
            return Some(self.players[i].1.clone());
        }

        None
    }
}
