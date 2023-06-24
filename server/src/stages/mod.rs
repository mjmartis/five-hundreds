mod aborted;
mod bidding;
mod lobby;

pub use self::aborted::Aborted;
pub use self::bidding::Bidding;
pub use self::lobby::Lobby;

use crate::api;
use crate::events;

use log::{error, info};

// The trait that each major "stage" (e.g. bidding, playing tricks) of a session should implement
// in order to be coordinated by the game engine.
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

// Common handling of an invalid response.
fn process_bad_step(
    stage: Box<dyn Stage>,
    players: &mut Vec<(events::ClientId, api::History)>,
    player_index: Option<usize>,
    clients: &events::ClientMap,
    client_id: &events::ClientId,
    step: &api::Step,
) -> Box<dyn Stage> {
    error!("[client {}] tried an invalid step: {:?}", client_id, step);
    clients.send_event(
        client_id,
        api::History {
            error: Some("Invalid step.".to_string()),
            ..player_index
                .map(|i| players[i].1.clone())
                .unwrap_or(Default::default())
        },
        api::CurrentState::Error,
    );

    stage
}
