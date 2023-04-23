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
        clients.send_event(
            client_id,
            // Include player history if this client is a valid player.
            player_index.map(|i| players[i].1.clone()),
            api::CurrentState::MatchAborted("Player left.".to_string()),
        );

        self
    }
}
