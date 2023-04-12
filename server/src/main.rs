pub mod api;

use api::Card;
use api::Face;
use api::State;
use api::Suit;
use api::SuitedCard;

fn main() {
    println!(
        "{:?}",
        State::WaitingForYourKitty(vec![
            Card::SuitedCard(SuitedCard {
                face: Face::Number(5),
                suit: Suit::Spades
            }),
            Card::SuitedCard(SuitedCard {
                face: Face::Ace,
                suit: Suit::Hearts
            }),
            Card::Joker
        ])
    );
}
