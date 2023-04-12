// Datatypes.

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
  Miz,
  OpenMiz,
  Pass,
}

#[derive(Debug)]
pub enum Face {
  Number(isize), // invariant: in [4..10]  
  Jack,
  Queen,
  King,
  Ace,
}

#[derive(Debug)]
pub struct SuitedCard {
  suit: Suit,
  face: Face,
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
