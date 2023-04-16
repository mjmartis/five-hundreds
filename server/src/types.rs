// Datatypes used in the server and server API.

#[derive(Debug)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(Debug)]
pub enum BidSuit {
    Suit(Suit),
    NoTrumps,
}

#[derive(Debug)]
pub enum Bid {
    Tricks(isize, BidSuit),
    Mis,
    OpenMis,
    Pass,
}

#[derive(Debug)]
pub enum Face {
    Number(isize), // Invariant: in [4..10].
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug)]
pub struct SuitedCard {
    pub face: Face,
    pub suit: Suit,
}

#[derive(Debug)]
pub enum Card {
    SuitedCard(SuitedCard),
    Joker,
}

// A card played on a turn. The joker is assigned its effective suit.
#[derive(Debug)]
pub enum Play {
    SuitedCard(SuitedCard),
    Joker(Suit),
}


