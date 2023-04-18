// When given a valid command, always responds with a dummy state.
// Try: wscat -c 127.0.0.1:8080 -x '"Poll"'

use std::env;

mod api;
mod events;
mod session;
mod types;
mod web_bridge;

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let rx = web_bridge::connect_bridge(addr);
    session::Session::new(rx).run_main_loop().await;
}
