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
    Tricks(isize, BidSuit),
    Mis,
    OpenMis,
    Pass,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Face {
    Number(isize), // Invariant: in [4..10].
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SuitedCard {
    pub face: Face,
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
