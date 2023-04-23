// The trait that each major "stage" (e.g. bidding, playing tricks) of a session should implement
// in order to be coordinated by the game engine.

use crate::api;
use crate::events;

pub trait Stage {
    // Accepts a step request from a client, and returns the next stage of the session. A stage
    // instance can return itself if the session stage hasn't changed.
    //
    // The initial arguments are "global" session information like the player statuses. The
    // player_index argument will be populated with the index of the current client in the players
    // list if they are a player.
    fn process_step(
        self: Box<Self>,
        players: &mut Vec<(events::ClientId, api::History)>,
        player_index: Option<usize>,
        clients: &events::ClientMap,
        client_id: &events::ClientId,
        step: &api::Step,
    ) -> Box<dyn Stage>;
}
