pub mod order_book;

use ordered_float::NotNan;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tungstenite::{connect, Message};
use url::Url;
use order_book::OrderBook;

#[derive(Debug, Deserialize)]
struct DepthUpdate {
    arg: Arg,
    data: Vec<Data>,
}

#[derive(Debug, Deserialize)]
struct Arg {
    channel: String,
    instId: String,
}

#[derive(Debug, Deserialize)]
struct Data {
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
    ts: String,
}

struct OrderBook {
    bids: BTreeMap<NotNan<f64>, f64>,
    asks: BTreeMap<NotNan<f64>, f64>,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn apply_update(&mut self, raw_message: &str, update: DepthUpdate) {
        println!("Applying update:\n{}", raw_message);

        if update.data.is_empty() {
            return;
        }

        for (p, q) in update.data[0].bids.iter() {
            let price: NotNan<f64> = NotNan::new(p.parse().unwrap()).unwrap();
            let qty: f64 = q.parse().unwrap();
            if qty == 0.0 {
                self.bids.remove(&price);
            } else {
                self.bids.insert(price, qty);
            }
        }

        for (p, q) in update.data[0].asks.iter() {
            let price: NotNan<f64> = NotNan::new(p.parse().unwrap()).unwrap();
            let qty: f64 = q.parse().unwrap();
            if qty == 0.0 {
                self.asks.remove(&price);
            } else {
                self.asks.insert(price, qty);
            }
        }

        if let Some(best_bid) = self.bids.keys().rev().next() {
            if let Some(best_ask) = self.asks.keys().next() {
                println!("Best Bid: {}, Best Ask: {}", best_bid, best_ask);
            }
        }
    }
}

pub fn run(ob: &mut OrderBook) {
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
        .write_message(Message::Text(subscribe_msg.to_string()))
        .expect("Failed to send subscribe message");

    println!("Subscribed to ETH-BTC order book");

    let order_book = Arc::new(Mutex::new(OrderBook::new()));

    loop {
        let msg = socket.read_message().expect("Error reading message");
        match msg {
            Message::Text(txt) => {
                let update: serde_json::Result<DepthUpdate> = serde_json::from_str(&txt);
                match update {
                    Ok(update) => {
                        let mut ob = order_book.lock().unwrap();
                        ob.apply_update(&txt, update);
                    }
                    Err(e) => {
                        println!("Failed to parse update: {}\nRaw: {}", e, txt);
                    }
                }
            }
            Message::Ping(p) => {
                socket
                    .write_message(Message::Pong(p))
                    .expect("Failed to send pong");
            }
            _ => {}
        }
    }
}
