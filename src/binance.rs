use std::sync::Arc;
use tokio::sync::Mutex;
use crate::order_book::{OrderBook, Side, PRICE_PRECISION, SIZE_PRECISION};
use crate::helpers::dot_trim;
use tokio_tungstenite::connect_async;
use serde_json::Value;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};

// TODO ping/pomg

#[allow(unused_assignments)]
pub async fn run(order_book: Arc<Mutex<OrderBook>>) -> Result<()> {
    // WebSocket
    let ws_url = "wss://stream.binance.com:9443/ws/ethbtc@depth@100ms";
    let (ws_stream, _) = connect_async(ws_url).await?;
    println!("BINANCE: WebSocket connected");
    let (mut write, mut read) = ws_stream.split();

    // Buffer updates before snapshot arrives
    let mut update_buffer: Vec<String> = Vec::new();
    let snapshot_received = Arc::new(AtomicBool::new(false));
    let flag = snapshot_received.clone();
    let mut last_update_id: i64 = 0;
    // Spawn a background task to fetch snapshot while buffering
    let ob = Arc::clone(&order_book);
    let snapshot_task = tokio::spawn(async move {
        let snapshot_url = "https://api.binance.com/api/v3/depth?symbol=ETHBTC&limit=1000";
        let resp = reqwest::get(snapshot_url).await.unwrap().json::<Value>().await.unwrap();
        last_update_id = resp["lastUpdateId"].as_i64().unwrap();
        println!("BINANCE: Snapshot loaded. lastUpdateId = {}", last_update_id);

        let mut b = ob.lock().await;
        if let Some(bids) = resp["bids"].as_array() {
            for bid in bids {
                let price: i32 = dot_trim(bid[0].to_string().replace('"', ""), PRICE_PRECISION).parse::<i32>().unwrap();
                let qty: i32 = dot_trim(bid[1].to_string().replace('"', ""), SIZE_PRECISION).parse::<i32>().unwrap();
                b.apply_update("BINANCE", Side::Bid, price, qty);
            }
        }
        if let Some(asks) = resp["asks"].as_array() {
            for ask in asks {
                let price: i32 = dot_trim(ask[0].to_string().replace('"', ""), PRICE_PRECISION).parse::<i32>().unwrap();
                let qty: i32 = dot_trim(ask[1].to_string().replace('"', ""), SIZE_PRECISION).parse::<i32>().unwrap();
                b.apply_update("BINANCE", Side::Ask, price, qty);
            }
        }

        flag.store(true, Ordering::SeqCst);
        last_update_id
    });

    // Step 2 — Buffer WebSocket updates until snapshot is ready
    while !snapshot_received.load(Ordering::SeqCst) {
        if let Some(msg) = read.next().await {
            let msg = msg?;
            if msg.is_text() {
                update_buffer.push(msg.into_text()?.to_string());
            }
        }
    }

    let last_update_id = snapshot_task.await.unwrap();

    // Step 3 — Process buffered updates
    for msg_text in update_buffer {
        let data: Value = serde_json::from_str(&msg_text)?;
        let _ob = Arc::clone(&order_book);
        if let Some(u) = data["u"].as_i64() {
            if u <= last_update_id {
                continue;
            }
            apply_binance_update(&_ob, &data).await;
        }
    }

    // Step 4 — Continue processing real-time updates
    let _ob = Arc::clone(&order_book);
    while let Some(msg) = read.next().await {
        let msg = msg?;
        match msg {
            Message::Text(txt) => {
                let data: Value = serde_json::from_str(&txt)?;
                if let Some(u) = data["u"].as_i64() {
                    if u <= last_update_id {
                        continue;
                    }
                    apply_binance_update(&_ob, &data).await;
                    //
                    // let mut __ob = _ob.lock().await;
                    // __ob.print();
                }
            }
            Message::Ping(payload) => {
                println!("BINANCE: Received Ping frame");
                write.send(Message::Pong(payload)).await?;
            }
            Message::Pong(_) => {
                println!("BINANCE: Received Pong frame");
            }
            _ => {}
        }
    }
    Ok::<_, anyhow::Error>(())
}

async fn apply_binance_update(book: &Arc<Mutex<OrderBook>>, data: &Value) {
    let mut b = book.lock().await;
    if let Some(bids) = data["b"].as_array() {
        for bid in bids {
            let price: i32 = dot_trim(bid[0].to_string().replace('"', ""), PRICE_PRECISION).parse::<i32>().unwrap();
            let qty: i32 = dot_trim(bid[1].to_string().replace('"', ""), SIZE_PRECISION).parse::<i32>().unwrap();
            b.apply_update("BINANCE", Side::Bid, price, qty);
        }
    }
    if let Some(asks) = data["a"].as_array() {
        for ask in asks {
            let price: i32 = dot_trim(ask[0].to_string().replace('"', ""), PRICE_PRECISION).parse::<i32>().unwrap();
            let qty: i32 = dot_trim(ask[1].to_string().replace('"', ""), SIZE_PRECISION).parse::<i32>().unwrap();
            b.apply_update("BINANCE", Side::Ask, price, qty);
        }
    }
}
