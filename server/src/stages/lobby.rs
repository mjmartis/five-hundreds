// The stage of the session where players are waiting to join a new game.

use super::aborted;

use crate::api;
use crate::events;

use log::info;

pub struct Lobby {}

impl super::Stage for Lobby {
    fn process_step(
        self: Box<Self>,
        players: &mut Vec<(events::ClientId, api::History)>,
        player_index: Option<usize>,
        clients: &events::ClientMap,
        client_id: &events::ClientId,
        step: &api::Step,
    ) -> Box<dyn super::Stage> {
        match &step {
            // A client is attempting to join.
            api::Step::Join(_) => {
                // Client is already in the player list.
                if let Some(i) = player_index {
                    clients.send_event(
                        client_id,
                        Some(players[i].1.clone()),
                        api::CurrentState::Excluded("Already joined.".to_string()),
                    );
                    info!(
                        "[client {}] excluded because they have already joined.",
                        client_id
                    );
                    return self;
                }

                // Player list is already full.
                if players.len() == 4 {
                    clients.send_event(
                        client_id,
                        None,
                        api::CurrentState::Excluded("Game ongoing.".to_string()),
                    );
                    info!("[client {}] excluded due to ongoing game.", client_id);
                    return self;
                }

                players.push((*client_id, Default::default()));
                info!("[client {}] joined.", client_id);

                // TODO send player joined messages to everyone.

                // All players newly joined.
                if players.len() == 4 {
                    info!("Starting match.");

                    // Tell clients that hands have been dealt.
                    // TODO: stop lying to them.
                    for (id, history) in players {
                        clients.send_event(id, Some(history.clone()), api::CurrentState::HandDealt);
                    }

                    // TODO return bidding state.
                    return Box::new(aborted::Aborted {});
                }

                self
            }

            // A client has left. This might end the game if they are an active player.
            // player.
            api::Step::Quit => {
                // Active player has left.
                if player_index.is_some() {
                    // TODO: send all clients goodbye messages.
                    info!("Player [client {}] left.", client_id);
                    return Box::new(aborted::Aborted {});
                } else {
                    info!("[client {}] tried to leave without joining.", client_id);
                    clients.send_event(
                        client_id,
                        None,
                        api::CurrentState::Error("Tried to leave without joining.".to_string()),
                    );
                }

                self
            }

            _ => {
                // TODO emit "unexpected step" error.
                self
            }
        }
    }
}
