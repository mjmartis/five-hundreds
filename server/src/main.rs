// When given a valid command, always responds with a dummy state.
// Try: wscat -c 127.0.0.1:8080 -x '"Poll"'

use std::collections::HashMap;
use std::env;

mod api;
mod events;
mod types;
mod web_bridge;

use crate::events::ClientPayload::StateSender;
use crate::events::ClientPayload::Step;

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut rx = web_bridge::connect_bridge(addr);
    let mut txs: HashMap<String, events::StateSender> = HashMap::new();

    loop {
        match rx.recv().await {
            // Should be the first message: a response channel.
            Some(events::ClientEvent {
                id,
                payload: StateSender(tx),
            }) => {
                txs.insert(id, tx);
            }

            // A regular step.
            Some(events::ClientEvent {
                id,
                payload: Step(_),
            }) => {
                process_step(&txs, &id);
            }

            // All communication is done.
            None => break,
        }
    }
}

// A placeholder; to be swapped for game engine logic.
fn process_step(txs: &HashMap<String, events::StateSender>, id: &str) {
    let Some(tx) = txs.get(id) else {
        // TODO log error: unseen client.
        return;
    };

    if tx.send(api::State::MatchWon(0)).is_err() {
        // TODO log error: can't communicate with client.
    }
}
