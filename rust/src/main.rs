pub mod api;

use api::Play;
use api::Suit;

fn main() {
    println!("{:?}", Play::Joker(Suit::Hearts));
}
