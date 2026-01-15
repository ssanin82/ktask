use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use crate::order_book::OrderBook;
use serde_json::json;
use tower_http::cors::CorsLayer;
use chrono::Utc;

pub async fn run_api_server(
    order_book: Arc<Mutex<OrderBook>>,
    tx: broadcast::Sender<String>,
) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/api/snapshot", get(get_snapshot))
        .route("/api/metrics", get(get_metrics))
        .route("/api/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(AppState {
            order_book,
            tx,
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:50051").await?;
    println!("API server listening on http://127.0.0.1:50051");
    println!("WebSocket endpoint: ws://127.0.0.1:50051/ws");
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    order_book: Arc<Mutex<OrderBook>>,
    tx: broadcast::Sender<String>,
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: AppState) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};
    
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();
    
    // Send initial snapshot
    {
        let ob = state.order_book.lock().await;
        let snapshot = ob.create_snapshot(50);
        if let Ok(json) = serde_json::to_string(&snapshot) {
            let _ = sender.send(Message::Text(json)).await;
        }
    }
    
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });
    
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

async fn get_snapshot(State(state): State<AppState>) -> impl IntoResponse {
    let ob = state.order_book.lock().await;
    let snapshot = ob.create_snapshot(50);
    axum::Json(snapshot)
}

async fn get_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let ob = state.order_book.lock().await;
    let snapshot = ob.create_snapshot(50);
    
    let metrics = json!({
        "timestamp": snapshot.timestamp,
        "spread": snapshot.spread,
        "best_bid": snapshot.best_bid,
        "best_ask": snapshot.best_ask,
        "mid_price": snapshot.mid_price,
        "total_bid_volume": snapshot.total_bid_volume,
        "total_ask_volume": snapshot.total_ask_volume,
        "imbalance": snapshot.imbalance,
        "vwap_bid": snapshot.vwap_bid,
        "vwap_ask": snapshot.vwap_ask,
        "depth_bid_5bps": snapshot.depth_bid_5bps,
        "depth_ask_5bps": snapshot.depth_ask_5bps,
        "depth_bid_10bps": snapshot.depth_bid_10bps,
        "depth_ask_10bps": snapshot.depth_ask_10bps,
        "update_counts": snapshot.update_counts,
    });
    
    axum::Json(metrics)
}

async fn health_check() -> impl IntoResponse {
    axum::Json(json!({
        "status": "ok",
        "timestamp": Utc::now()
    }))
}
