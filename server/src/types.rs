// Datatypes used in the server and server API.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BidSuit {
    Suit(Suit),
    NoTrumps,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Bid {
    Tricks(usize, BidSuit), // Invariant: first element in [6..10].
    Mis,
    OpenMis,
    Pass,
}

#[derive(Clone, Debug, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SuitedCard {
    // The number or face on the card. Invariant: in [4..14], with ace represented by 14.
    pub face: usize,
    pub suit: Suit,
}

#[derive(Clone, Debug, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
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
