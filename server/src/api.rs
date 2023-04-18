// The API used for web clients and the server to communicate.

use crate::types;

use serde::{Deserialize, Serialize};

// The actions a player can take.
#[derive(Debug, Serialize, Deserialize)]
pub enum Step {
    // Degenerate case: just ask to see state.
    Poll,

    // Ask to join.
    Join(isize), // The index of the team to join (i.e. in [0, 1]).

    // Make a bid.
    MakeBid(types::Bid),

    // Take or leave kitty cards.
    UseKitty(Vec<types::Card>), // Your selected hand. Invariant: length of 10.

    // Announce the suit of the joker in your hand.
    AnnounceJokerSuit(types::Suit),

    // Choose a card (and possibly the suit of the joker) to play.
    MakePlay(types::Play),

    // Exit the match early.
    Quit,
}

// The state that the session can be in.
#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    // You or another player have just joined.
    // Player count, your index and your resume token are stored in context struct.
    // TODO: transmit e.g. player names.
    PlayerJoined,

    // You have been rejected (e.g. because a game is ongoing).
    Excluded(String), // Reason.

    // Your hand has been dealt.
    // types::Card vector of length 10 stored in context struct.
    HandDealt,

    // Your turn to bid.
    WaitingForYourBid(Vec<types::Bid>), // Bids available to you.

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
    WaitingForYourKitty(Vec<types::Card>), // Invariant: length of 3.

    // When they must choose how to use the kitty.
    WaitingForTheirKitty,

    // When you must announce the suit of your joker.
    WaitingForYourJokerSuit,

    // When they must announce the suit of their joker.
    WaitingForTheirJokerSuit,

    // When they announce the suit of their joker.
    // types::Suit stored in context struct.
    JokerSuitAnnounced,

    // Waiting for you to choose a card to play.
    WaitingForYourPlay(Vec<types::Play>),

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
    Error(String), // Reason.
}

// Static info.

// Background information about the lobby.
#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyContext {
    // Number of players currently joined.
    pub players_count: isize,

    // Your index in the player list.
    pub your_player_index: isize,

    // Your index in the team list (i.e. in [0, 1]).
    pub your_team_index: isize,
}

// Background information about the match.
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchContext {
    // Game history. TODO make more sophisticated.
    //   (team 1 score delta, team 1 score total,
    //    team 2 score delta, team 2 score total).
    pub past_games: Vec<(isize, isize, isize, isize)>,
}

// Background information about the bidding.
#[derive(Debug, Serialize, Deserialize)]
pub struct BiddingContext {
    // The last bids made by each player. Ordered from player 1 to player 4.
    pub bids: Vec<Option<types::Bid>>, // Invariant: length of 4.

    pub current_bidder_index: isize,
}

// Background information about the bid that won.
#[derive(Debug, Serialize, Deserialize)]
pub struct WinningBidContext {
    pub winning_bidder_index: isize,

    pub winning_bid: types::Bid, // Invariant: not a Pass.
}

// Background information about the tricks being played.
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaysContext {
    // The joker suit, if fixed.
    pub joker_suit: Option<types::Suit>,

    // The number of tricks you have won.
    pub your_tricks_count: isize,

    // The number of tricks they have won.
    pub their_tricks_count: isize,

    // The number of cards in each player's hand.
    pub hand_sizes: Vec<isize>, // Invariant: length of 4.

    // The previous trick, if there was one. Listed in order from player 1 to
    // player 4. Inner Option is to support e.g. mis bids, where one player
    // doesn't play.
    pub previous_trick: Option<Vec<Option<types::Play>>>,

    // The ongoing trick. Listed in order from player 1 to player 4.
    pub current_trick: Vec<Option<types::Play>>,

    // Index in the player list of the currently-playing player.
    pub currently_playing_player_index: isize,
}

// Background information about the current game (i.e. the current bidding,
// bidding-won, hands played cycle).
#[derive(Debug, Serialize, Deserialize)]
pub struct GameContext {
    // The cards in your hand.
    pub hand: Vec<types::Card>,

    pub bidding_context: BiddingContext,

    pub winning_bid_context: Option<WinningBidContext>,

    pub plays_context: Option<PlaysContext>,
}

// Background information about the session. Sub-contexts are populated as they
// become valid.
#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    pub lobby_context: LobbyContext,

    pub match_context: Option<MatchContext>,

    pub game_context: Option<GameContext>,
}
