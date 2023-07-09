// The stage of the game where one player makes use of the kitty and possibly
// declares the suit of the Joker.

use crate::api;
use crate::events;
use crate::types::*;

use log::error;
use std::collections::HashSet;
use std::debug_assert;

use super::Stage;

pub struct BidWon {
    winning_bidder_index: usize,
    kitty: Vec<Card>,
}

impl BidWon {
    pub fn new(
        players: &mut Vec<(events::ClientId, api::History)>,
        clients: &events::ClientMap,
        winning_bidder_index: usize,
        winning_bid: Bid,
        kitty: Vec<Card>,
    ) -> Self {
        // Notify players that the bid has been won.
        for (id, history) in players.iter_mut() {
            history.game_history.as_mut().unwrap().winning_bid_history =
                Some(api::WinningBidHistory {
                    winning_bidder_index,
                    winning_bid,
                    kitty: None,
                    discarded: None,
                });

            clients.send_event(id, history.clone(), api::CurrentState::BidWon);
        }

        // Assign the kitty to the winning bidder.
        unwrap_winning_bid_history(&mut players[winning_bidder_index].1).kitty =
            Some(kitty.clone());

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
            kitty,
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
        // Bail with an error response if this isn't a player.
        let Some(index) = super::reject_nonplayer(player_index, clients, client_id, step) else { return self; };

        match step {
            api::Step::DiscardCards(cards) => {
                // Player isn't the bid winner.
                if index != self.winning_bidder_index {
                    error!(
                        "[client {}] tried to discard cards without winning the bid",
                        client_id
                    );
                    clients.send_event(
                        client_id,
                        api::History {
                            error: Some("You don't have the kitty.".to_string()),
                            ..players[index].1.clone()
                        },
                        api::CurrentState::WaitingForTheirKitty,
                    );

                    return self;
                }

                // The set of cards chosen to discard.
                let discarded = cards.iter().copied().collect::<HashSet<_>>();

                // Verify three cards have been discarded.
                if discarded.len() != 3 {
                    error!(
                        "[client {}] tried to discard the wrong number of cards",
                        client_id
                    );
                    clients.send_event(
                        client_id,
                        api::History {
                            error: Some(
                                "You tried to discard the wrong number of cards.".to_string(),
                            ),
                            ..players[index].1.clone()
                        },
                        api::CurrentState::WaitingForYourKitty,
                    );

                    return self;
                }

                // The combination of the winner's hand and the kitty.
                let mut held_cards = HashSet::<_>::new();
                held_cards.extend(unwrap_game_history(&mut players[index].1).hand.clone());
                held_cards.extend(self.kitty.clone());

                // Verify the discards are from the player's hand or the kitty.
                if !discarded.is_subset(&held_cards) {
                    error!(
                        "[client {}] tried to discard cards they don't hold",
                        client_id
                    );
                    clients.send_event(
                        client_id,
                        api::History {
                            error: Some("You tried to discard cards you don't hold.".to_string()),
                            ..players[index].1.clone()
                        },
                        api::CurrentState::WaitingForYourKitty,
                    );

                    return self;
                }

                // The bid winner has chosen legitimate cards to discard. Update the player's hand.
                let new_hand = held_cards
                    .difference(&discarded)
                    .copied()
                    .collect::<Vec<_>>();
                debug_assert!(new_hand.len() == 10);
                players[index].1.game_history.as_mut().unwrap().hand = new_hand;
                unwrap_winning_bid_history(&mut players[index].1).kitty = None;
                unwrap_winning_bid_history(&mut players[index].1).discarded =
                    Some(discarded.iter().copied().collect::<Vec<_>>());

                // TODO: handle joker declaration and actual play.
                for (id, history) in players.iter() {
                    clients.send_event(id, history.clone(), api::CurrentState::WaitingForTheirPlay);
                }
            }

            _bad_step => {
                super::process_bad_step(
                    players,
                    player_index,
                    clients,
                    client_id,
                    step,
                    if index == self.winning_bidder_index {
                        api::CurrentState::WaitingForYourKitty
                    } else {
                        api::CurrentState::WaitingForTheirKitty
                    },
                    "after bid won",
                );
            }
        }

        self
    }
}

// Convenience functions to extract mutable sub-histories.
fn unwrap_game_history(history: &mut api::History) -> &mut api::GameHistory {
    history.game_history.as_mut().unwrap()
}

fn unwrap_winning_bid_history(history: &mut api::History) -> &mut api::WinningBidHistory {
    unwrap_game_history(history)
        .winning_bid_history
        .as_mut()
        .unwrap()
}
