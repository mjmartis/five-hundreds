// The stage of the session where players are waiting to join a new game.

use super::bidding;
use super::Stage;

use crate::api;
use crate::events;

use log::info;

pub struct Lobby {
    game_index: usize,
}

impl Lobby {
    pub fn new(game_index: usize) -> Self {
        Lobby { game_index }
    }
}

impl Stage for Lobby {
    fn process_step(
        self: Box<Self>,
        players: &mut Vec<(events::ClientId, api::History)>,
        player_index: Option<usize>,
        clients: &events::ClientMap,
        client_id: &events::ClientId,
        step: &api::Step,
    ) -> Box<dyn Stage> {
        match &step {
            // A client is attempting to join.
            api::Step::Join(_) => {
                // Client is already in the player list.
                if let Some(i) = player_index {
                    clients.send_event(
                        client_id,
                        api::History {
                            error: Some("Already joined.".to_string()),
                            ..players[i].1.clone()
                        },
                        api::CurrentState::PlayerJoined,
                    );
                    info!(
                        "[client {}] excluded because they have already joined.",
                        client_id
                    );
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
                            ..Default::default()
                        }),
                        ..Default::default()
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
            }

            // A client has made a step that isn't valid in the lobby.
            _bad_step => {
                super::process_bad_step(
                    players,
                    player_index,
                    clients,
                    client_id,
                    step,
                    if player_index.is_some() {
                        api::CurrentState::PlayerJoined
                    } else {
                        api::CurrentState::Error
                    },
                    "in the lobby",
                );
            }
        }

        self
    }
}
