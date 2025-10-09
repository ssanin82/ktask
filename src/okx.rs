use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tungstenite::{Message};
use crate::order_book::{OrderBook, Side, PRICE_PRECISION, SIZE_PRECISION};
use crate::helpers::dot_trim;

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
    prev_seq_id: i64
}

pub fn run(order_book: Arc<Mutex<OrderBook>>) {
// pub fn main() {
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
                        for (p, q, _x, _y) in update.data[0].bids.iter() {
                            let price: i32 = dot_trim(p.clone(), PRICE_PRECISION).parse::<i32>().unwrap();
                            let qty: i32 = dot_trim(q.clone(), SIZE_PRECISION).parse::<i32>().unwrap();
                            ob.add_level(Side::Bid, price, qty);
                        }

                        for (p, q, _x, _y) in update.data[0].asks.iter() {
                            let price: i32 = dot_trim(p.clone(), PRICE_PRECISION).parse::<i32>().unwrap();
                            let qty: i32 = dot_trim(q.clone(), SIZE_PRECISION).parse::<i32>().unwrap();
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
