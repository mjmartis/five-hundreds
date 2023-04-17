use futures_util::SinkExt;
use futures_util::StreamExt;
use std::env;
use std::io;

use tokio::sync::mpsc;
use tokio_tungstenite as tokio_ws2;
use tokio_ws2::tungstenite as ws2;

type Sender = mpsc::UnboundedSender<ws2::Message>;
type Receiver = mpsc::UnboundedReceiver<ws2::Message>;

// Returns a sender that transmits messages to one WebSocket client, and a receiver that receives
// messages from that same client. Forms the basis of logic that will eventually handle game engine
// <-> web client communication.
fn init_echo_socket(
    websocket: tokio_ws2::WebSocketStream<tokio::net::TcpStream>,
) -> (Sender, Receiver) {
    let (mut write, mut read) = websocket.split();

    // Thread consuming messages transmitted from the server.
    let (out_tx, mut out_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            if let Some(msg) = out_rx.recv().await {
                if write.send(msg).await.is_err() {
                    println!("Failed to reply to client");
                    return;
                }
            } else {
                println!("Connection closed by server");
                return;
            }
        }
    });

    // Thread transmitting messages received through the web socket.
    let (in_tx, in_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            if let Some(result) = read.next().await {
                if result.is_ok() && in_tx.send(result.unwrap()).is_err() {
                    println!("Failed to consume client message");
                    return;
                }
            } else {
                println!("Connection closed by client");
                return;
            }
        }
    });

    (out_tx, in_rx)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Listen for TCP requests incoming to the given address.
    let listener = tokio::net::TcpListener::bind(addr.clone()).await?;
    println!("Listening on: {addr}");

    loop {
        // Establish the TCP connection.
        let Ok((stream, client_addr)) = listener.accept().await else { continue };

        // Establish the WebSocket connection.
        let Ok(websocket) = tokio_ws2::accept_async(stream).await else {continue };
        println!("New WebSocket connection: {client_addr}");

        tokio::spawn(async move {
            // Get handles to listen and reply to the given client.
            let (out_tx, mut in_rx) = init_echo_socket(websocket);

            loop {
                if let Some(msg) = in_rx.recv().await {
                    if !msg.is_text() && !msg.is_binary() {
                        continue;
                    }

                    if let Err(mpsc::error::SendError(_)) = out_tx.send(msg) {
                        println!("Failed to communicate with sending thread");
                        break;
                    }
                } else {
                    println!("Failed to communicate with receiving thread");
                    break;
                }
            }
        });
    }
}
