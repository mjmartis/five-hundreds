// The stage of the session where players are waiting to join a new game.

use super::bidding;

use crate::api;
use crate::events;

use log::{error, info};

pub struct Lobby {
    game_index: usize,
}

impl Lobby {
    pub fn new(game_index: usize) -> Self {
        Lobby { game_index }
    }
}

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
                        api::History {
                            excluded_reason: Some("Already joined.".to_string()),
                            ..players[i].1.clone()
                        },
                        api::CurrentState::Excluded,
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
                        api::History {
                            excluded_reason: Some("Game ongoing.".to_string()),
                            ..Default::default()
                        },
                        api::CurrentState::Excluded,
                    );
                    info!("[client {}] excluded due to ongoing game.", client_id);
                    return self;
                }

                // Note: starts with incorrect player count to match the other histories that are
                // now out-of-date.
                players.push((
                    (*client_id.clone()).to_string(),
                    api::History {
                        lobby_history: Some(api::LobbyHistory {
                            player_count: players.len(),
                            your_player_index: players.len(),
                            your_team_index: players.len() % 2,
                        }),
                        match_history: Some(api::MatchHistory {
                            past_games: Vec::new(),
                            match_aborted_reason: None,
                            winning_team_index: None,
                        }),
                        game_history: None,
                        excluded_reason: None,
                        error: None,
                    },
                ));
                info!("[client {}] joined.", client_id);

                // Let players know another has joined.
                for (id, history) in &mut *players {
                    // Invariant: all instances added to the players list have
                    // lobby history populated.
                    history.lobby_history.as_mut().unwrap().player_count += 1;
                    clients.send_event(id, history.clone(), api::CurrentState::PlayerJoined);
                }

                // All players newly joined.
                if players.len() == 4 {
                    info!("Starting match.");
                    return Box::new(bidding::Bidding::new(players, clients, self.game_index % 4));
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
                    if let Some(i) = player_index {
                        api::History {
                            error: Some("Invalid step in the lobby stage.".to_string()),
                            ..players[i].1.clone()
                        }
                    } else {
                        Default::default()
                    },
                    api::CurrentState::Error,
                );

                self
            }
        }
    }
}
