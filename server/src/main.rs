use std::env;
use std::net;
use std::thread;
use std::sync::mpsc;

use tungstenite as ws2;

// Send WebSocket messages from the given TCP stream through the given channel. Forms the basis of
// logic that will send messages from web clients to the game engine.
fn channel_ws_msgs(stream: Result<net::TcpStream, std::io::Error>, tx: mpsc::Sender<ws2::Message>) {
    if stream.is_err() {
        return;
    }

    let mut websocket = match ws2::accept(stream.unwrap()) {
        Err(_) => return,
        Ok(v) => v,
    };

    loop {
        let msg = match websocket.read_message() {
            Err(_) => break,
            Ok(v) => v,
        };

        if !msg.is_text() {
            continue;
        }

        tx.send(msg).unwrap();
    }
}

fn main () {
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create a channel and print its messages on another thread.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for m in rx {
            println!("{}", m);
        }
    });

    // Send messages from all incoming WebSocket connections to the channel.
    let server = net::TcpListener::bind(addr).unwrap();
    for stream in server.incoming() {
        let tx2 = tx.clone();
        thread::spawn(move || channel_ws_msgs(stream, tx2));
    }
}
