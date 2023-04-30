// The stage of the game where players make bids for one game.

use crate::api;
use crate::events;
use crate::types::*;

use rand::seq::SliceRandom;
use std::debug_assert;

pub struct Bidding {
    first_bidder_index: usize,

    hands: Vec<Vec<Card>>,
    kitty: Vec<Card>,

    // The latest bids for each player. Used to determine which bids can be
    // made for each player.
    prev_bids: Vec<Option<Bid>>,

    highest_bid: Option<Bid>,
}

impl Bidding {
    pub fn new(
        players: &mut Vec<(events::ClientId, api::History)>,
        clients: &events::ClientMap,
        first_bidder_index: usize,
    ) -> Self {
        debug_assert!(players.len() == 4);

        // Populate and shuffle deck.
        let mut deck = (5..15)
            .flat_map(|face| {
                vec![
                    Card::SuitedCard(SuitedCard {
                        face,
                        suit: Suit::Spades,
                    }),
                    Card::SuitedCard(SuitedCard {
                        face,
                        suit: Suit::Clubs,
                    }),
                    Card::SuitedCard(SuitedCard {
                        face,
                        suit: Suit::Diamonds,
                    }),
                    Card::SuitedCard(SuitedCard {
                        face,
                        suit: Suit::Hearts,
                    }),
                ]
            })
            .chain(vec![
                Card::SuitedCard(SuitedCard {
                    face: 4,
                    suit: Suit::Diamonds,
                }),
                Card::SuitedCard(SuitedCard {
                    face: 4,
                    suit: Suit::Hearts,
                }),
                Card::Joker,
            ])
            .collect::<Vec<_>>();
        deck.shuffle(&mut rand::thread_rng());

        // Deal hands.
        let chunks: Vec<&[Card]> = deck.chunks(10).collect();
        let hands: Vec<Vec<Card>> = chunks[0..4].iter().map(|h| h.to_vec()).collect();
        debug_assert!(chunks.len() == 5);

        let kitty = chunks[4].to_vec();
        debug_assert!(kitty.len() == 3);

        let new = Bidding {
            first_bidder_index,
            hands: hands.clone(),
            kitty,
            prev_bids: vec![None; 4],
            highest_bid: None,
        };

        for (i, (id, history)) in players.iter_mut().enumerate() {
            // Clear old history and populate a new history with the new hand.
            history.match_history.past_games = Vec::new();
            history.game_history = Some(api::GameHistory {
                hand: hands[i].clone(),
                bidding_history: api::BiddingHistory {
                    bids: vec![None; 4],
                    current_bidder_index: first_bidder_index,
                },
                winning_bid_history: None,
                plays_history: None,
            });

            // Send off hands and bidding cues.
            clients.send_event(id, Some(history.clone()), api::CurrentState::HandDealt);
            clients.send_event(
                id,
                Some(history.clone()),
                if i == first_bidder_index {
                    api::CurrentState::WaitingForYourBid(new.available_bids(i))
                } else {
                    api::CurrentState::WaitingForTheirBid
                },
            );
        }

        new
    }
}

impl Bidding {
    // Returns the bids that the given player can take at this point in the
    // bidding (i.e. applying mis rules).
    fn available_bids(&self, player_index: usize) -> Vec<Bid> {
        debug_assert!(player_index < 4);

        // Can't bid again if you've passed.
        let mut bids: Vec<Bid> = vec![Bid::Pass];
        if self.prev_bids[player_index] == Some(Bid::Pass) {
            return bids;
        }

        // Can't bid miseres until everyone has had a chance to bid.
        let can_bid_mis = self
            .prev_bids
            .iter()
            .filter(|b| b.is_some())
            .collect::<Vec<_>>()
            .len()
            == 4;

        let mut cur_bid = self.highest_bid.unwrap_or(Bid::Pass);
        while let Some(bid) = next_bid(cur_bid) {
            match bid {
                Bid::Mis | Bid::OpenMis if can_bid_mis => {
                    bids.push(bid);
                }
                bid @ Bid::Tricks(_, _) => {
                    bids.push(bid);
                }
                _ => {}
            }

            cur_bid = bid;
        }

        bids
    }
}

impl super::Stage for Bidding {
    fn process_step(
        self: Box<Self>,
        players: &mut Vec<(events::ClientId, api::History)>,
        player_index: Option<usize>,
        clients: &events::ClientMap,
        client_id: &events::ClientId,
        step: &api::Step,
    ) -> Box<dyn super::Stage> {
        self
    }
}

// Returns the next highest bid.
fn next_bid(bid: Bid) -> Option<Bid> {
    match bid {
        Bid::Pass => Some(Bid::Tricks(6, BidSuit::Suit(Suit::Spades))),

        // Mis is worth 270 pts.
        Bid::Tricks(8, BidSuit::Suit(Suit::Clubs)) => Some(Bid::Mis),
        Bid::Mis => Some(Bid::Tricks(8, BidSuit::Suit(Suit::Diamonds))),

        Bid::Tricks(count, suit) => {
            let new_count = if suit == BidSuit::NoTrumps {
                count + 1
            } else {
                count
            };
            if new_count == 11 {
                return Some(Bid::OpenMis);
            }

            let new_suit = match suit {
                BidSuit::Suit(Suit::Spades) => BidSuit::Suit(Suit::Clubs),
                BidSuit::Suit(Suit::Clubs) => BidSuit::Suit(Suit::Diamonds),
                BidSuit::Suit(Suit::Diamonds) => BidSuit::Suit(Suit::Hearts),
                BidSuit::Suit(Suit::Hearts) => BidSuit::NoTrumps,
                BidSuit::NoTrumps => BidSuit::Suit(Suit::Spades),
            };

            Some(Bid::Tricks(new_count, new_suit))
        }

        // No way to outbid open mis.
        Bid::OpenMis => None,
    }
}
