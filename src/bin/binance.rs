use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tungstenite::{connect, Message};
use url::Url;
use ordered_float::NotNan;

#[derive(Debug, Deserialize)]
struct Snapshot {
    lastUpdateId: u64,
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
}

#[derive(Debug, Deserialize)]
struct DepthUpdate {
    #[serde(rename = "U")]
    first_update_id: u64,
    #[serde(rename = "u")]
    final_update_id: u64,
    b: Vec<(String, String)>, // bids
    a: Vec<(String, String)>, // asks
}

struct OrderBook {
    bids: BTreeMap<NotNan<f64>, f64>, // price -> qty
    asks: BTreeMap<NotNan<f64>, f64>,
    last_update_id: u64,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: 0,
        }
    }

    fn apply_snapshot(&mut self, snapshot: Snapshot) {
        self.last_update_id = snapshot.lastUpdateId;
        self.bids.clear();
        self.asks.clear();
        for (p, q) in snapshot.bids {
            self.bids.insert(p.parse().unwrap(), q.parse().unwrap());
        }
        for (p, q) in snapshot.asks {
            self.asks.insert(p.parse().unwrap(), q.parse().unwrap());
        }
        println!("Snapshot loaded: lastUpdateId = {}", self.last_update_id);
    }

    fn apply_update(&mut self, update: DepthUpdate) {
        if update.final_update_id <= self.last_update_id {
            return; // outdated
        }

        if update.first_update_id > self.last_update_id + 1 {
            println!("Out of sync: fetching snapshot again...");
            return;
        }

        for (p, q) in update.b {
            let price: NotNan<f64> = p.parse().unwrap();
            let qty: f64 = q.parse().unwrap();
            if qty == 0.0 {
                self.bids.remove(&price);
            } else {
                self.bids.insert(price, qty);
            }
        }

        for (p, q) in update.a {
            let price: NotNan<f64> = p.parse().unwrap();
            let qty: f64 = q.parse().unwrap();
            if qty == 0.0 {
                self.asks.remove(&price);
            } else {
                self.asks.insert(price, qty);
            }
        }

        self.last_update_id = update.final_update_id;

        if let Some(best_bid) = self.bids.keys().rev().next() {
            if let Some(best_ask) = self.asks.keys().next() {
                println!("Best Bid: {}, Best Ask: {}", best_bid, best_ask);
            }
        }
    }
}

fn get_snapshot(symbol: &str) -> Snapshot {
    let url = format!(
        "https://api.binance.com/api/v3/depth?symbol={}&limit=1000",
        symbol
    );
    let resp = reqwest::blocking::get(&url).expect("Failed to fetch snapshot");
    let text = resp.text().expect("Failed to read response text");
    println!("Snapshot response:\n{}", text); // <-- debug print
    serde_json::from_str(&text).expect("Failed to parse snapshot")
}

fn main() {
    let symbol = "ETHBTC".to_string();
    let order_book = Arc::new(Mutex::new(OrderBook::new()));

    // Get initial snapshot
    {
        let snapshot = get_snapshot(&symbol);
        let mut ob = order_book.lock().unwrap();
        ob.apply_snapshot(snapshot);
    }

    // WebSocket
    let ws_url = format!("wss://stream.binance.com:9443/ws/{}@depth@100ms", symbol.to_lowercase());
    let (mut socket, _response) = tungstenite::connect(ws_url).expect("Can't connect");

    println!("WebSocket connected");

    loop {
        let msg = socket.read_message().expect("Error reading message");
        if let Message::Text(txt) = msg {
            println!("{}", &txt);
            let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
            if let Some(data) = v.get("data") {
                if let Ok(update) = serde_json::from_value::<DepthUpdate>(data.clone()) {
                    let mut ob = order_book.lock().unwrap();
                    ob.apply_update(update);
                }
            }
        }
    }
}
