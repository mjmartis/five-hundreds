mod aborted;
mod bid_won;
mod bidding;
mod lobby;

pub use self::aborted::Aborted;
pub use self::bid_won::BidWon;
pub use self::bidding::Bidding;
pub use self::lobby::Lobby;

use crate::api;
use crate::events;

use log::error;

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

// Common logic to return an error response to a client that isn't a player.
fn reject_nonplayer(
    player_index: Option<usize>,
    clients: &events::ClientMap,
    client_id: &events::ClientId,
    step: &api::Step,
) -> Option<usize> {
    if player_index.is_some() {
        return player_index;
    }

    if let api::Step::Join(_) = step {
        clients.send_event(
            client_id,
            api::History {
                error: Some("Match has started.".to_string()),
                ..Default::default()
            },
            api::CurrentState::Excluded,
        );
    } else {
        clients.send_event(
            client_id,
            api::History {
                error: Some("You are not a player in this game.".to_string()),
                ..Default::default()
            },
            api::CurrentState::Error,
        );
    }

    return None;
}

// Common logic to return an error message for an unexpected step.
fn process_bad_step(
    players: &mut Vec<(events::ClientId, api::History)>,
    player_index: Option<usize>,
    clients: &events::ClientMap,
    client_id: &events::ClientId,
    step: &api::Step,
    state: api::CurrentState,
    stage_name: &str,
) {
    error!(
        "[client {}] tried an invalid step {}: {:?}",
        client_id, stage_name, step
    );

    clients.send_event(
        client_id,
        api::History {
            error: Some(format!("Invalid step {}", stage_name)),
            ..player_index
                .map(|i| players[i].1.clone())
                .unwrap_or(Default::default())
        },
        state,
    );
}
