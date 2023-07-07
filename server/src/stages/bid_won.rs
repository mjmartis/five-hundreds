// The stage of the game where one player makes use of the kitty and possibly
// declares the suit of the Joker.

use crate::api;
use crate::events;
use crate::types::*;

use log::{error, info};
use std::collections::HashSet;
use std::debug_assert;

use super::Stage;

pub struct BidWon {
    winning_bidder_index: usize,
    hands: Vec<Vec<Card>>,
    kitty: Vec<Card>,
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
            .kitty = Some(kitty.clone());

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
        match &step {
            api::Step::DiscardCards(cards) => {
                // Client isn't even a player.
                if player_index.is_none() {
                    clients.send_event(
                        client_id,
                        api::History {
                            error: Some("You are not a player in this game.".to_string()),
                            ..Default::default()
                        },
                        api::CurrentState::Error,
                    );

                    return self;
                }

                let index = player_index.unwrap();

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

                // Verify three cards have been discarded.
                if cards.len() != 3 {
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
                held_cards.extend(self.hands[index].clone());
                held_cards.extend(self.kitty.clone());

                // Verify the discards are from the player's hand or the kitty.
                if !(cards
                    .iter()
                    .copied()
                    .collect::<HashSet<_>>()
                    .is_subset(&held_cards))
                {
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
                for card in cards {
                    let removed = held_cards.remove(&card);
                    debug_assert!(removed);
                }
                players[index].1.game_history.as_mut().unwrap().hand =
                    held_cards.into_iter().collect::<Vec<_>>();
                players[index]
                    .1
                    .game_history
                    .as_mut()
                    .unwrap()
                    .winning_bid_history
                    .as_mut()
                    .unwrap()
                    .kitty = Some(cards.clone());

                // TODO: handle joker declaration and actual play.
                for (id, history) in players.iter() {
                    clients.send_event(id, history.clone(), api::CurrentState::WaitingForTheirPlay);
                }
            }

            bad_step => {
                error!(
                    "[client {}] tried an invalid step after bid won: {:?}",
                    client_id, bad_step
                );

                let state = match player_index {
                    None => api::CurrentState::Error,
                    Some(i) if i == self.winning_bidder_index => {
                        api::CurrentState::WaitingForYourKitty
                    }
                    _ => api::CurrentState::WaitingForTheirKitty,
                };

                clients.send_event(
                    client_id,
                    api::History {
                        error: Some("Invalid step after bid won.".to_string()),
                        ..player_index
                            .map(|i| players[i].1.clone())
                            .unwrap_or(Default::default())
                    },
                    state,
                );
            }
        }

        self
    }
}
