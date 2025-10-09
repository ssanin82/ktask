pub mod order_book;
pub mod helpers;

use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tungstenite::{Message};
use order_book::{OrderBook, Side};
use helpers::dot_trim;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DepthUpdate {
    arg: Arg,
    action: String,
    data: Vec<Data>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Arg {
    channel: String,
    #[serde(rename = "instId")]
    inst_id: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Data {
    bids: Vec<(String, String, String, String)>,
    asks: Vec<(String, String, String, String)>,
    ts: String,
    checksum: i64,
    #[serde(rename = "seqId")]
    seq_id: u64,
    #[serde(rename = "prevSeqId")]
    prev_seq_id: u64
}

// pub fn run(ob: &mut OrderBook) {
pub fn main() {
    let (mut socket, _response) = tungstenite::connect("wss://ws.okx.com:8443/ws/v5/public").expect("Can't connect");

    let subscribe_msg = serde_json::json!({
        "op": "subscribe",
        "args": [
            {
                "channel": "books",
                "instId": "ETH-BTC"
            }
        ]
    });

    socket
        .send(Message::Text(subscribe_msg.to_string()))
        .expect("Failed to send subscribe message");

    println!("Subscribed to ETH-BTC order book");

    let order_book = Arc::new(Mutex::new(OrderBook::new()));

    loop {
        let msg = socket.read().expect("Error reading message");
        match msg {
            Message::Text(txt) => {
                let update: serde_json::Result<DepthUpdate> = serde_json::from_str(&txt);
                match update {
                    Ok(update) => {
                        let mut ob = order_book.lock().unwrap();
                        if update.data.is_empty() {
                            continue;
                        }
                        // ETHBTC tick size precision: 5)
                        // ETHBTC lot size precision: 6)
                        for (p, q, _x, _y) in update.data[0].bids.iter() {
                            let price: u32 = dot_trim(p.clone(), 5).parse::<u32>().unwrap();
                            let qty: u64 = dot_trim(q.clone(), 6).parse::<u64>().unwrap();
                            ob.add_level(Side::Bid, price, qty);
                        }

                        for (p, q, _x, _y) in update.data[0].asks.iter() {
                            let price: u32 = dot_trim(p.clone(), 5).parse::<u32>().unwrap();
                            let qty: u64 = dot_trim(q.clone(), 6).parse::<u64>().unwrap();
                            ob.add_level(Side::Ask, price, qty);
                        }

                        // ob.print();
                    }
                    Err(e) => {
                        println!("Failed to parse update: {}\nRaw: {}", e, txt);
                    }
                }
            }
            Message::Ping(p) => {
                socket
                    .send(Message::Pong(p))
                    .expect("Failed to send pong");
            }
            _ => {}
        }
    }
}
