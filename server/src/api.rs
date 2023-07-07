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

    // You have been rejected (e.g. because a game is ongoing). The reason is
    // stored in the lobby history struct.
    Excluded,

    // Your hand has been dealt.
    // types::Card vector of length 10 stored in history struct.
    HandDealt,

    // Your turn to bid. Your bid options are stored in the bidding history
    // struct.
    WaitingForYourBid,

    // Waiting for another player to bid.
    // Current bidder stored in history struct.
    WaitingForTheirBid,

    // Another player has bid. Their bid is stored in history struct.
    PlayerBid,

    // A player (possibly you) has won the bid.
    // Winning player stored in history struct.
    BidWon,

    // When you must choose how to use the kitty (i.e. you have won the bid).
    // Kitty is stored in the bidding won history struct.
    WaitingForYourKitty,

    // When they must choose how to use the kitty.
    WaitingForTheirKitty,

    // When you must announce the suit of your joker.
    WaitingForYourJokerSuit,

    // When they must announce the suit of their joker.
    WaitingForTheirJokerSuit,

    // When they announce the suit of their joker.
    // types::Suit stored in history struct.
    JokerSuitAnnounced,

    // Waiting for you to choose a card to play. Your options are stored in the
    // plays history struct.
    WaitingForYourPlay,

    // Waiting for another player to play.
    // Current playing player (and trick so far) stored in plays history struct.
    WaitingForTheirPlay,

    // You or another player has won the trick. The winning player is stored in
    // the plays history struct.
    TrickWon,

    // Your or the other team have won the game. The index of the winning team
    // is stored in the match history struct.
    GameWon,

    // The new scores have been included in the history struct.
    ScoresUpdated,

    // A team has won the entire match. The index of the winning team is stored
    // in the match history struct.
    MatchWon,

    // The match has unexpectedly ended (e.g. a player has left). The reason is
    // stored in the history struct.
    MatchAborted,

    // Some other in-game error (e.g. tried to play an invalid card). The
    // reason is stored in the history struct.
    Error,
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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MatchHistory {
    // Game history. TODO make more sophisticated.
    //   (team 1 score delta, team 1 score total,
    //    team 2 score delta, team 2 score total).
    pub past_games: Vec<(isize, isize, isize, isize)>,

    // The index of the team that won the entire match, if the match is over.
    pub winning_team_index: Option<usize>,

    // The reason the match was aborted (e.g. a player left), if it has been.
    pub match_aborted_reason: Option<String>,
}

// Background information about the bidding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiddingHistory {
    // Possible bids you can make, if it is your turn to bid.
    pub bid_options: Option<Vec<types::Bid>>,

    // The last bids made by each player. Ordered from player 1 to player 4.
    pub bids: Vec<Option<types::Bid>>, // Invariant: length of 4.

    pub current_bidder_index: usize,
}

// Background information about the bid that won.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinningBidHistory {
    pub winning_bidder_index: usize,

    pub winning_bid: types::Bid, // Invariant: not a Pass.

    // The cards in your kitty, if you won the bidding.
    pub kitty: Option<Vec<types::Card>>,
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

    // The index of the player who won the last trick, if there is one.
    pub previous_trick_winner: Option<usize>,

    // The ongoing trick. Listed in order from player 1 to player 4.
    pub current_trick: Vec<Option<types::Play>>,

    // Index in the player list of the currently-playing player.
    pub currently_playing_player_index: usize,

    // Your possible plays, if it is your turn to play a card.
    pub play_options: Option<Vec<types::Play>>,
}

// Background information about the current game (i.e. the current bidding,
// bidding-won, hands played cycle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameHistory {
    // The cards in your hand.
    pub hand: Vec<types::Card>,

    pub bidding_history: Option<BiddingHistory>,

    pub winning_bid_history: Option<WinningBidHistory>,

    pub plays_history: Option<PlaysHistory>,
}

// Background information about the session. Sub-structs are populated as they
// become valid.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct History {
    pub lobby_history: Option<LobbyHistory>,

    pub match_history: Option<MatchHistory>,

    pub game_history: Option<GameHistory>,

    // The reason you have been excluded from the game, if there is one.
    pub excluded_reason: Option<String>,

    // Some other error, if there is one.
    pub error: Option<String>,
}

// Top level state information sent to the client.
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub state: CurrentState,
    pub history: History,
}
