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
}

impl Bidding {
    pub fn new(
        players: &mut Vec<(events::ClientId, api::History)>,
        clients: &events::ClientMap,
        first_bidder_index: usize,
    ) -> Self {
        debug_assert!(players.len() == 4);

        // Shuffle deck.
        let mut deck = deck();
        deck.shuffle(&mut rand::thread_rng());

        // Deal hands.
        let chunks: Vec<&[Card]> = deck.chunks(10).collect();
        let hands: Vec<Vec<Card>> = chunks[0..4].iter().map(|h| h.to_vec()).collect();
        let kitty = chunks[4].to_vec();

        for (i, (id, history)) in players.iter_mut().enumerate() {
            // Update history to include new hand.
            history.game_history = Some(api::GameHistory {
                hand: hands[i].clone(),
                bidding_history: api::BiddingHistory {
                    bids: [None, None, None, None].to_vec(),
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
                    api::CurrentState::WaitingForYourBid(Vec::new())
                } else {
                    api::CurrentState::WaitingForTheirBid
                },
            );
        }

        Bidding {
            first_bidder_index,
            hands,
            kitty,
        }
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

// Populate a vector with the cards from a standard 500s game.
fn deck() -> Vec<Card> {
    let mut deck = Vec::new();

    for face in 5..14 {
        for suit in [Suit::Spades, Suit::Clubs, Suit::Diamonds, Suit::Hearts] {
            deck.push(Card::SuitedCard(SuitedCard { suit, face }));
        }
    }
    deck.push(Card::SuitedCard(SuitedCard { suit: Suit::Diamonds, face: 4 }));
    deck.push(Card::SuitedCard(SuitedCard { suit: Suit::Hearts, face: 4 }));
    deck.push(Card::Joker);

    deck
}
