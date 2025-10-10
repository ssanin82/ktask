use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tungstenite::{Message};
use crate::order_book::{OrderBook, Side, PRICE_PRECISION, SIZE_PRECISION};
use crate::helpers::dot_trim;

#[derive(Debug, Deserialize)]
struct Snapshot {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,
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

fn get_snapshot(symbol: &str) -> Snapshot {
    let url = format!(
        "https://api.binance.com/api/v3/depth?symbol={}&limit=1000",
        symbol
    );
    let resp = reqwest::blocking::get(&url).expect("Failed to fetch snapshot");
    let text = resp.text().expect("Failed to read response text");
    // println!("Snapshot response:\n{}", text); // <-- debug print
    serde_json::from_str(&text).expect("Failed to parse snapshot")
}

pub fn run(order_book: Arc<Mutex<OrderBook>>) {
    let symbol = "ETHBTC".to_string();
    let mut last_update_id: u64;

    // WebSocket
    let ws_url = format!("wss://stream.binance.com:9443/ws/{}@depth@100ms", symbol.to_lowercase());
    let (mut socket, _response) = tungstenite::connect(ws_url).expect("Can't connect");
    println!("WebSocket connected");

    // Get initial snapshot
    {
        let snapshot = get_snapshot(&symbol);
        let mut ob = order_book.lock().unwrap();
        last_update_id = snapshot.last_update_id;
        for (p, q) in snapshot.bids {
            let price: i32 = dot_trim(p.clone(), PRICE_PRECISION).parse::<i32>().unwrap();
            let qty: i32 = dot_trim(q.clone(), SIZE_PRECISION).parse::<i32>().unwrap();
            ob.apply_update("BINANCE", Side::Bid, price, qty);
        }
        for (p, q) in snapshot.asks {
            let price: i32 = dot_trim(p.clone(), PRICE_PRECISION).parse::<i32>().unwrap();
            let qty: i32 = dot_trim(q.clone(), SIZE_PRECISION).parse::<i32>().unwrap();
            ob.apply_update("BINANCE", Side::Ask, price, qty);
        }
        ob.print();
    }

    loop {
        let msg = socket.read().expect("Error reading message");
        match msg {
            Message::Text(txt) => {
                let update: serde_json::Result<DepthUpdate> = serde_json::from_str(&txt);
                match update {
                    Ok(update) => {
                        let mut ob = order_book.lock().unwrap();
                        if update.final_update_id <= last_update_id {
                            println!(
                                "Outdated seq id: {} < {}",
                                update.final_update_id,
                                last_update_id
                            );
                            continue;
                        }
                        if update.first_update_id > last_update_id + 1 {
                            panic!(
                                "Out of sync: fetching snapshot again... ({} - {})",
                                update.first_update_id,
                                last_update_id
                            );
                        }
                        // println!("{}", &txt);
                        for (p, q) in update.b {
                            let price: i32 = dot_trim(p.clone(), PRICE_PRECISION).parse::<i32>().unwrap();
                            let qty: i32 = dot_trim(q.clone(), SIZE_PRECISION).parse::<i32>().unwrap();
                            ob.apply_update("BINANCE", Side::Bid, price, qty);
                        }
                        for (p, q) in update.a {
                            let price: i32 = dot_trim(p.clone(), PRICE_PRECISION).parse::<i32>().unwrap();
                            let qty: i32 = dot_trim(q.clone(), SIZE_PRECISION).parse::<i32>().unwrap();
                            ob.apply_update("BINANCE", Side::Ask, price, qty);
                        }
                        last_update_id = update.final_update_id;
                        // ob.print();
                    }
                    Err(e) => {
                        println!("Failed to parse update: {}\nRaw: {}", e, txt);
                    }
                }
            }
            Message::Ping(p) => {
                println!("Received ping, sending pong");
                socket
                    .send(Message::Pong(p))
                    .expect("Failed to send pong");
            }
            Message::Pong(_) => {
                println!("Received pong");
            }
            Message::Close(frame) => {
                println!("WebSocket closed: {:?}", frame);
                break;
            }
            _ => {}
        }
    }
}
