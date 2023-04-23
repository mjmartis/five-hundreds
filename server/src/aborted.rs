// The stage of the game when an unrecoverable error (e.g. player has quit) error has been
// encountered.

use crate::api;
use crate::events;
use crate::stages;
use crate::types;

pub struct Aborted {}

impl stages::Stage for Aborted {
    // Always send the error state back.
    fn process_step(
        self: Box<Self>,
        players: &mut Vec<(events::ClientId, api::History)>,
        player_index: Option<usize>,
        clients: &events::ClientMap,
        client_id: &events::ClientId,
        _step: &api::Step,
    ) -> Box<dyn stages::Stage> {
        clients.send_event(
            client_id,
            // Include player history if this client is a valid player.
            if let Some(i) = player_index {
                Some(players[i].1.clone())
            } else {
                None
            },
            api::CurrentState::MatchAborted("Player left.".to_string()),
        );

        return self;
    }
}
