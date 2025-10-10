use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use serde_json::json;
use anyhow::Result;
use crate::order_book::{OrderBook, Side, PRICE_PRECISION, SIZE_PRECISION};
use crate::helpers::dot_trim;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Arg {
    channel: String,
    #[serde(rename = "instId")]
    inst_id: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct DepthUpdate {
    arg: Arg,
    action: String,
    data: Vec<Data>,
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
    prev_seq_id: i64,
}

pub async fn run(order_book: Arc<Mutex<OrderBook>>) -> Result<()> {
    let symbol = "ETH-BTC";

    let url = "wss://ws.okx.com:8443/ws/v5/public";
    println!("OKX: Connecting to {url} ...");
    let (ws_stream, _) = connect_async(url).await?;
    println!("OKX: Connected.");

    let (mut write, mut read) = ws_stream.split();

    let sub_msg = json!({
        "op": "subscribe",
        "args": [{ "channel": "books", "instId": symbol }]
    });
    write
        .send(Message::Text(sub_msg.to_string().into()))
        .await
        .expect("Failed to send subscription");
    println!("OKX: Subscribed to {symbol} order book.");

    // Read WebSocket messages
    let ob_clone = Arc::clone(&order_book);
    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Text(txt) => {
                // println!("OKX {}", txt);
                let update: serde_json::Result<DepthUpdate> = serde_json::from_str(&txt);
                match update {
                    Ok(update) => {
                        if update.data.is_empty() {
                            continue;
                        }
                        let mut ob = ob_clone.lock().await;
                        for (p, q, _x, _y) in update.data[0].bids.iter() {
                            let price: i32 = dot_trim(p.to_string(), PRICE_PRECISION).parse::<i32>().unwrap();
                            let qty: i32 = dot_trim(q.to_string(), SIZE_PRECISION).parse::<i32>().unwrap();
                            ob.apply_update("OKX", Side::Bid, price, qty);
                        }
                        for (p, q, _x, _y) in update.data[0].asks.iter() {
                            let price: i32 = dot_trim(p.to_string(), PRICE_PRECISION).parse::<i32>().unwrap();
                            let qty: i32 = dot_trim(q.to_string(), SIZE_PRECISION).parse::<i32>().unwrap();
                            ob.apply_update("OKX", Side::Ask, price, qty);
                        }
                        // ob.print();
                    }
                    Err(_e) => {
                        // XXX just ignore for now, it is mainly subs ack
                        // println!("OKX: Failed to parse update: {}\nRaw: {}", e, txt);
                    }
                }
            }
            Message::Ping(p) => {
                println!("OKX PING");
                write.send(Message::Pong(p)).await?;
            }
            _ => {}
        }
    }

    Ok::<_, anyhow::Error>(())
}
