use reqwest::blocking::Client;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::time::Duration;

// For websocket
use tungstenite::protocol::Message;
use tungstenite::connect;
use url::Url;

#[derive(Debug, Deserialize)]
struct DepthSnapshot {
    lastUpdateId: u64,
    bids: Vec<(String, String)>, // price, qty
    asks: Vec<(String, String)>,
}

#[derive(Debug, Deserialize)]
struct DepthUpdate {
    e: String,   // event type
    E: u64,      // event time
    s: String,   // symbol
    U: u64,      // first update id in event
    u: u64,      // final update id in event
    b: Vec<(String, String)>,
    a: Vec<(String, String)>,
}

fn fetch_rest_snapshot(symbol: &str, limit: u16) -> Result<DepthSnapshot, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let base = "https://api.binance.com";
    let url = format!("{}/api/v3/depth?symbol={}&limit={}", base, symbol, limit);

    let resp = client.get(&url).send()?;
    if !resp.status().is_success() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }

    // Binance returns bids/asks as arrays of [price, qty]; serde will decode into tuple vecs
    let snapshot: DepthSnapshot = resp.json()?;
    Ok(snapshot)
}

fn run_rest_mode() -> Result<(), Box<dyn Error>> {
    // Binance symbols are uppercase for REST
    let symbol = "ETHBTC";
    let limit = 20u16;

    println!("Fetching REST depth snapshot for {} (top {})...", symbol, limit);
    let snapshot = fetch_rest_snapshot(symbol, limit)?;

    println!("lastUpdateId: {}", snapshot.lastUpdateId);
    println!("--- BIDS (top {}) ---", snapshot.bids.len());
    for (i, (price, qty)) in snapshot.bids.iter().enumerate() {
        println!("{:>2}. price: {:>12}, qty: {}", i + 1, price, qty);
    }

    println!("--- ASKS (top {}) ---", snapshot.asks.len());
    for (i, (price, qty)) in snapshot.asks.iter().enumerate() {
        println!("{:>2}. price: {:>12}, qty: {}", i + 1, price, qty);
    }

    Ok(())
}

fn run_ws_mode() -> Result<(), Box<dyn Error>> {
    // WebSocket symbol is lowercase
    let stream = "ethbtc@depth20@100ms";
    // let stream = "ethbtc@bookTicker";
    let url_str = format!("wss://stream.binance.com:9443/ws/{}", stream);
    let url = Url::parse(&url_str)?;

    println!("Connecting to {}", url);
    let (mut socket, response) = connect(url)?;
    println!("Connected. HTTP status: {}", response.status());

    println!("Receiving depth updates for {}, press Ctrl+C to stop.", stream);

    loop {
        let msg = socket.read_message()?;
        match msg {
            Message::Text(txt) => {
                // parse JSON to DepthUpdate
                match serde_json::from_str::<DepthUpdate>(&txt) {
                    Ok(update) => {
                        println!(
                            "\nEvent: {} (event time: {}, U: {}, u: {})",
                            update.e, update.E, update.U, update.u
                        );

                        println!("Top {} bids:", update.b.len());
                        for (i, (price, qty)) in update.b.iter().enumerate() {
                            println!("{:>2}. {} @ {}", i + 1, qty, price);
                        }

                        println!("Top {} asks:", update.a.len());
                        for (i, (price, qty)) in update.a.iter().enumerate() {
                            println!("{:>2}. {} @ {}", i + 1, qty, price);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse depth update JSON: {} -- raw: {}", e, txt);
                    }
                }
            }
            Message::Ping(p) => {
                socket.send(Message::Pong(p))?;
            }
            Message::Pong(_) => {}
            Message::Binary(_) => {
                // Binance uses text JSON for depth so this is unlikely
            }
            Message::Close(frame) => {
                println!("WebSocket closed: {:?}", frame);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} [rest|ws]", program);
    eprintln!("  rest - one-off REST snapshot (GET /api/v3/depth?limit=20)");
    eprintln!("  ws   - live blocking WebSocket depth stream (wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms)");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    if args.len() != 2 {
        print_usage(program);
        std::process::exit(1);
    }

    let mode = args[1].as_str();
    let res = match mode {
        "rest" => run_rest_mode(),
        "ws" => run_ws_mode(),
        _ => {
            print_usage(program);
            std::process::exit(1);
        }
    };

    if let Err(e) = res {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
