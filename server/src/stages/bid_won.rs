// The stage of the game where one player makes use of the kitty and possibly
// declares the suit of the Joker.

use crate::api;
use crate::events;
use crate::types::*;

use log::{error, info};
use std::debug_assert;

use super::Stage;

pub struct BidWon {
    winning_bidder_index: usize,
    hands: Vec<Vec<Card>>,
}

impl BidWon {
    pub fn new(
        players: &mut Vec<(events::ClientId, api::History)>,
        clients: &events::ClientMap,
        winning_bidder_index: usize,
        winning_bid: Bid,
        hands: Vec<Vec<Card>>,
        kitty: Vec<Card>,
    ) -> Self {
        // Notify players that the bid has been won.
        for (id, history) in players.iter_mut() {
            history.game_history.as_mut().unwrap().winning_bid_history =
                Some(api::WinningBidHistory {
                    winning_bidder_index,
                    winning_bid,
                    kitty: None,
                });

            clients.send_event(id, history.clone(), api::CurrentState::BidWon);
        }

        // Assign the kitty to the winning bidder.
        players[winning_bidder_index]
            .1
            .game_history
            .as_mut()
            .unwrap()
            .winning_bid_history
            .as_mut()
            .unwrap()
            .kitty = Some(kitty);

        // Now notify players that the kitty needs to be used.
        for (i, (id, history)) in players.iter().enumerate() {
            clients.send_event(
                id,
                history.clone(),
                if i == winning_bidder_index {
                    api::CurrentState::WaitingForYourKitty
                } else {
                    api::CurrentState::WaitingForTheirKitty
                },
            );
        }

        BidWon {
            winning_bidder_index,
            hands,
        }
    }
}

impl Stage for BidWon {
    fn process_step(
        self: Box<Self>,
        players: &mut Vec<(events::ClientId, api::History)>,
        player_index: Option<usize>,
        clients: &events::ClientMap,
        client_id: &events::ClientId,
        step: &api::Step,
    ) -> Box<dyn Stage> {
        self
    }
}
