// The API used for web clients and the server to communicate.

use crate::types;

use serde::{Deserialize, Serialize};

// The actions a player can take.
#[derive(Debug, Serialize, Deserialize)]
pub enum Step {
    // Degenerate case: just ask to see state.
    Poll,

    // Ask to join.
    Join(usize), // The index of the team to join (i.e. in [0, 1]).

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

// The most recent state that the session is in.
#[derive(Debug, Serialize, Deserialize)]
pub enum CurrentState {
    // You or another player have just joined.
    // Player count, your index and your resume token are stored in history struct.
    // TODO: transmit e.g. player names.
    PlayerJoined,

    // You have been rejected (e.g. because a game is ongoing).
    Excluded(String), // Reason.

    // Your hand has been dealt.
    // types::Card vector of length 10 stored in history struct.
    HandDealt,

    // Your turn to bid.
    WaitingForYourBid(Vec<types::Bid>), // Bids available to you.

    // Waiting for another player to bid.
    // Current bidder stored in history struct.
    WaitingForTheirBid,

    // Another player has bid.
    // Their bid stored in history struct.
    TheyBid(usize), // The player who made their bid.

    // A player (possibly you) has won the bid.
    // Winning player stored in history struct.
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
    // types::Suit stored in history struct.
    JokerSuitAnnounced,

    // Waiting for you to choose a card to play.
    WaitingForYourPlay(Vec<types::Play>),

    // Waiting for another player to play.
    // Current playing player (and trick so far) stored in history struct.
    WaitingForTheirPlay,

    // You or another player has won the trick.
    TrickWon(usize), // Index of winning player.

    // Your or the other team have won the game.
    GameWon(usize), // Index of winning team (i.e. in [0, 1]).

    // The new scores have been included in the history struct.
    ScoresUpdated,

    // A team has won the entire match.
    MatchWon(usize), // Index of winning team (i.e. in [0, 1]).

    // The match has unexpectedly ended (e.g. a player has left).
    MatchAborted(String), // Reason.

    // Some other in-game error (e.g. tried to play an invalid card).
    Error(String), // Reason.
}

// Static info.

// Background information about the lobby.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LobbyHistory {
    // Number of players currently joined.
    pub player_count: usize,

    // Your index in the player list.
    pub your_player_index: usize,

    // Your index in the team list (i.e. in [0, 1]).
    pub your_team_index: usize,
}

// Background information about the match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchHistory {
    // Game history. TODO make more sophisticated.
    //   (team 1 score delta, team 1 score total,
    //    team 2 score delta, team 2 score total).
    pub past_games: Vec<(isize, isize, isize, isize)>,
}

// Background information about the bidding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiddingHistory {
    // The last bids made by each player. Ordered from player 1 to player 4.
    pub bids: Vec<Option<types::Bid>>, // Invariant: length of 4.

    pub current_bidder_index: usize,
}

// Background information about the bid that won.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinningBidHistory {
    pub winning_bidder_index: usize,

    pub winning_bid: types::Bid, // Invariant: not a Pass.
}

// Background information about the tricks being played.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaysHistory {
    // The joker suit, if fixed.
    pub joker_suit: Option<types::Suit>,

    // The number of tricks you have won.
    pub your_tricks_count: usize,

    // The number of tricks they have won.
    pub their_tricks_count: usize,

    // The number of cards in each player's hand.
    pub hand_sizes: Vec<usize>, // Invariant: length of 4.

    // The previous trick, if there was one. Listed in order from player 1 to
    // player 4. Inner Option is to support e.g. mis bids, where one player
    // doesn't play.
    pub previous_trick: Option<Vec<Option<types::Play>>>,

    // The ongoing trick. Listed in order from player 1 to player 4.
    pub current_trick: Vec<Option<types::Play>>,

    // Index in the player list of the currently-playing player.
    pub currently_playing_player_index: usize,
}

// Background information about the current game (i.e. the current bidding,
// bidding-won, hands played cycle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameHistory {
    // The cards in your hand.
    pub hand: Vec<types::Card>,

    pub bidding_history: BiddingHistory,

    pub winning_bid_history: Option<WinningBidHistory>,

    pub plays_history: Option<PlaysHistory>,
}

// Background information about the session. Sub-structs are populated as they become valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub lobby_history: LobbyHistory,

    pub match_history: MatchHistory,

    pub game_history: Option<GameHistory>,
}

// Top level state information sent to the client.
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub state: CurrentState,

    // Only populated if the client is a player.
    pub history: Option<History>,
}
