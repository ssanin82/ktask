use tungstenite::{client::IntoClientRequest, Message};
use tungstenite::client_tls;
use url::Url;
use std::net::TcpStream;
use rustls::crypto::aws_lc_rs;
use rustls::crypto::CryptoProvider;
use clap::Parser;
use std::thread;
use std::time::Duration;
use std::fmt;
use serde::Deserialize;
use helpers::dot_trim;

const BINANCE_URL: &str = "wss://stream.binance.com:9443/ws/ethbtc@depth20";
const OKX_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";

const OKX_SUBS: &str = r#"{
    "op": "subscribe",
    "args": [
        {
            "channel": "books",
            "instType": "SPOT",
            "instId": "ETH-BTC",
            "sz": "10"
        }
    ]
}"#;

#[derive(Debug)]
struct LevelUpdate {
    price: i32,
    size: i64
}

#[derive(Debug)]
struct OrderBookUpdate {
    bids: Vec<LevelUpdate>,
    asks: Vec<LevelUpdate>,
}

#[derive(Deserialize)]
struct RawOrderBookBinance {
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}

#[derive(Deserialize)]
struct RawOrderBookOkx {
    bids: Vec<[String; 4]>,
    asks: Vec<[String; 4]>,
}

#[derive(Deserialize)]
struct DataWrapperOkx {
    data: Option<Vec<RawOrderBookOkx>>,
}

enum Xch {
    Binance,
    Okx,
}

impl fmt::Display for Xch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Xch::Binance => write!(f, "BINANCE"),
            Xch::Okx => write!(f, "OKX"),
        }
    }
}

impl Xch {
    fn parse_msg(&self, msg: String) {
        println!("RECEIVED: {}", msg);
        match self {
            Xch::Binance => {
                // ETHBTC tick size precision: 5)
                // ETHBTC lot size precision: 4)
                let raw: RawOrderBookBinance = serde_json::from_str(msg.as_str()).unwrap();
                let bids = raw.bids.iter().take(10).map(|b| LevelUpdate {
                    price: dot_trim(b[0].clone(), 5).parse::<i32>().unwrap(),
                    size:  dot_trim(b[1].clone(), 4).parse::<i64>().unwrap(),
                }).collect();
                let asks = raw.asks.iter().take(10).map(|a| LevelUpdate {
                    price: dot_trim(a[0].clone(), 5).parse::<i32>().unwrap(),
                    size:  dot_trim(a[1].clone(), 4).parse::<i64>().unwrap(),
                }).collect();
                let order_book = OrderBookUpdate { bids, asks };
                println!("{:#?}", order_book);
            }
            Xch::Okx => {
                // ETHBTC tick size precision: 5)
                // ETHBTC lot size precision: 6)
                let parsed: DataWrapperOkx = serde_json::from_str(msg.as_str()).unwrap();
                if let Some(data_list) = parsed.data {
                    if let Some(raw) = data_list.first() {
                        // Safely process first entry
                        let bids: Vec<LevelUpdate> = raw.bids.iter().take(10).map(|b| LevelUpdate {
                            price: dot_trim(b[0].clone(), 5).parse::<i32>().unwrap(),
                            size:  dot_trim(b[1].clone(), 6).parse::<i64>().unwrap(),
                        }).collect();

                        let asks: Vec<LevelUpdate> = raw.asks.iter().take(10).map(|a| LevelUpdate {
                            price: dot_trim(a[0].clone(), 5).parse::<i32>().unwrap(),
                            size:  dot_trim(a[1].clone(), 6).parse::<i64>().unwrap(),
                        }).collect();

                        let order_book = OrderBookUpdate { bids, asks };
                        println!("{:#?}", order_book);
                    } else {
                        println!("⚠️ 'data' field is present but empty");
                    }
                } else {
                    println!("❌ 'data' field is missing");
                }
            }
        }
    }
}

fn run_price_stream(xch: Xch, _url: &str, _subs: Option<&str>) {
    println!("Getting price stream for: {}", xch.to_string());

    let _ = CryptoProvider::install_default(aws_lc_rs::default_provider());

    let url = Url::parse(_url).unwrap();
    let req = url.as_str().into_client_request().unwrap();
    let domain = url.domain().unwrap();
    let port = url.port().unwrap();

    let tcp = TcpStream::connect((domain, port)).unwrap();
    let (mut socket, _) = client_tls(req, tcp).unwrap();
    println!("Connected to feed: {}", _url);

    match _subs {
        Some(ss) => {
            let _ = socket.send(Message::Text(String::from(ss)));
            println!("Subscribed: {}", ss);
        }
        None => println!("No subscription needed"),
    }

    loop {
        match socket.read() {
            Ok(Message::Text(txt)) => {
                // println!("{} {}", xch.to_string(), txt);
                xch.parse_msg(txt);
            }
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "server")]
#[command(about = "Reads host and port arguments", long_about = None)]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    #[arg(long, default_value_t = 8080)]
    port: i32,
}

fn main() {
    let args = Args::parse();
    println!("Host: {}", args.host);
    println!("Port: {}", args.port);

    // let feed_bnc = thread::spawn(move || {
    //     run_price_stream(Xch::Binance, BINANCE_URL, None);
    // });
    let feed_okx = thread::spawn(move || {
        run_price_stream(Xch::Okx, OKX_URL, Some(OKX_SUBS));
    });
    for i in 1..=5 {
        println!("Main thread working: {}", i);
        thread::sleep(Duration::from_millis(3000));
    }
    // feed_bnc.join().unwrap();
    feed_okx.join().unwrap();
    println!("Done!");
}
