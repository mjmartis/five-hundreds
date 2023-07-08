// The stage of the game when an unrecoverable error (e.g. player has quit) error has been
// encountered.

use crate::api;
use crate::events;

pub struct Aborted {}

impl super::Stage for Aborted {
    // Always send the error state back.
    fn process_step(
        self: Box<Self>,
        players: &mut Vec<(events::ClientId, api::History)>,
        player_index: Option<usize>,
        clients: &events::ClientMap,
        client_id: &events::ClientId,
        _step: &api::Step,
    ) -> Box<dyn super::Stage> {
        // Include player history if this client is a valid player.
        let history = if let Some(i) = player_index {
            api::History {
                match_history: players[i]
                    .1
                    .match_history
                    .clone()
                    .map(|h| api::MatchHistory {
                        match_aborted_reason: Some("Player left.".to_string()),
                        ..h
                    }),
                ..players[i].1.clone()
            }
        } else {
            api::History {
                error: Some("Match aborted".to_string()),
                ..Default::default()
            }
        };

        clients.send_event(client_id, history, api::CurrentState::MatchAborted);

        self
    }
}
