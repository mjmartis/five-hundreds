// Datatypes used in the server and server API.

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BidSuit {
    Suit(Suit),
    NoTrumps,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Bid {
    Tricks(usize, BidSuit),  // Invariant: first element in [6..10].
    Mis,
    OpenMis,
    Pass,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SuitedCard {
    // The number or face on the card. Invariant: in [4..13], with ace represented by 14.
    pub face: usize,
    pub suit: Suit,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Card {
    SuitedCard(SuitedCard),
    Joker,
}

// A card played on a turn. The joker is assigned its effective suit.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Play {
    SuitedCard(SuitedCard),
    Joker(Suit),
}
