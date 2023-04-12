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

// Messages.

// The actions a player can take.
#[derive(Debug)]
pub enum Step {
    // Degenerate case: just ask to see state.
    Poll,

    // Ask to join.
    Join,

    // Make a bid.
    MakeBid(Bid),

    // Take or leave kitty cards.
    UseKitty(Vec<Card>), // Your selected hand. Invariant: length of 10.

    // Announce the suit of the joker in your hand.
    AnnounceJokerSuit(Suit),

    // Choose a card (and possibly the suit of the joker) to play.
    MakePlay(Play),

    // Exit the match early.
    Leave,
}

// The state that the session can be in.
#[derive(Debug)]
pub enum State {
    // You or another player have just joined.
    // (player count, your index) stored in context struct.
    PlayerJoined,

    // You have been rejected (e.g. because a game is ongoing).
    Excluded(String), // Reason.

    // Your hand has been dealt.
    // Card vector of length 10 stored in context struct.
    HandDealt,

    // Your turn to bid.
    WaitingForYourBid(Vec<Bid>), // Bids available to you.

    // Waiting for another player to bid.
    // Current bidder stored in context struct.
    WaitingForTheirBid,

    // Another player has bid.
    // Their bid stored in context struct.
    TheyBid(isize), // The player who made their bid.

    // A player (possibly you) has won the bid.
    // Winning player stored in context struct.
    BidWon,

    // When you must choose how to use the kitty (i.e. you have won the bid).
    WaitingForYourKitty(Vec<Card>), // Invariant: length of 3.

    // When they must choose how to use the kitty.
    WaitingForTheirKitty,

    // When you must announce the suit of your joker.
    WaitingForYourJokerSuit,

    // When they must announce the suit of their joker.
    WaitingForTheirJokerSuit,

    // When they announce the suit of their joker.
    // Suit stored in context struct.
    JokerSuitAnnounced,

    // Waiting for you to choose a card to play.
    WaitingForYourPlay(Vec<Play>),

    // Waiting for another player to play.
    // Current playing player (and trick so far) stored in context struct.
    WaitingForTheirPlay,

    // You or another player has won the trick.
    TrickWon(isize), // Index of winning player.

    // Your or the other team have won the game.
    GameWon(isize), // Index of winning team (i.e. in [0, 1]).

    // The new scores have been included in the context struct.
    ScoresUpdated,

    // A team has won the entire match.
    MatchWon(isize), // Index of winning team (i.e. in [0, 1]).

    // The match has unexpectedly ended (e.g. a player has left).
    MatchAborted(String), // Reason.

    // Some other in-game error (e.g. tried to play an invalid card).
    MatchError(String), // Reason.
}
