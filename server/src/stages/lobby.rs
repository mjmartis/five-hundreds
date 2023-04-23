// The stage of the session where players are waiting to join a new game.

use super::aborted;

use crate::api;
use crate::events;

use log::{error, info};

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

                // Note: starts with incorrect player count to match the other histories that are
                // now out-of-date.
                players.push((
                    *client_id,
                    api::History {
                        lobby_history: api::LobbyHistory {
                            player_count: players.len(),
                            your_player_index: players.len(),
                            your_team_index: players.len() % 2,
                        },
                        ..Default::default()
                    },
                ));
                info!("[client {}] joined.", client_id);

                // Let players know another has joined.
                for (id, history) in &mut *players {
                    history.lobby_history.player_count += 1;
                    clients.send_event(id, Some(history.clone()), api::CurrentState::PlayerJoined);
                }

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
            api::Step::Quit => {
                // Active player has left.
                if player_index.is_some() {
                    // Let everyone know the game can't continue.
                    for (id, history) in players {
                        clients.send_event(
                            id,
                            Some(history.clone()),
                            api::CurrentState::MatchAborted("Player left".to_string()),
                        );
                    }

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

            // A client has made a step that isn't valid in the lobby.
            bad_step => {
                error!(
                    "[client {}] tried an invalid step in the lobby stage: {:?}",
                    client_id, bad_step
                );
                clients.send_event(
                    client_id,
                    player_index.map(|i| players[i].1.clone()),
                    api::CurrentState::Error("Invalid step in the lobby stage.".to_string()),
                );

                self
            }
        }
    }
}
