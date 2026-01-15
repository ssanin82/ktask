

# Order Book Aggregator with HFT Visualization

A high-performance order book aggregator written in Rust that consolidates ETH/BTC order book data from multiple exchanges (Binance and OKX) and provides a professional HFT-style web dashboard for visualization.

## Features

- **Real-time Order Book Aggregation**: Consolidates order book data from Binance and OKX
- **High-Performance Backend**: Written in Rust with integer-based precision for speed and accuracy
- **Professional Web Dashboard**: React-based frontend with HFT-style visualizations
- **Multiple Interfaces**: gRPC, WebSocket, and REST API support
- **Advanced Analytics**: VWAP, depth analysis, order book imbalance, and more
- **Exchange Comparison**: Side-by-side analysis of exchange contributions

## Data Sources

Aggregates spot ETHBTC prices from 2 sources:
- Binance
- OKX

### Price/Volume Calculations
For storing the order book, no floating point numbers are used. Only integers - for precision and speed.

Prices/sizes are converted to float strings at the consumer component side for display. Price precision chosen is 5, size precision chosen is 6. This is derived from current tick sizes and lot sizes in selected exchanges for BTCUSDT:

- Binance Reference Data
  - URL: https://api.binance.com/api/v3/exchangeInfo
  - ETHBTC lot size as of today:   0.0001  (size precision: 4)
  - ETHBTC tick size as of today: 0.00001  (price precision: 5)
- OKX Reference Data
  - URL: https://www.okx.com/api/v5/public/instruments?instType=SPOT
  - ETH-BTC lot size as of today: 0.000001 (size precision: 6)
  - ETH-BTC tick size as of today: 0.00001 (price precision: 5)

The chosen price/size precision is the maximum of the corresponding precisions on all 2 exchanges. It would have been impossible to use this technique with more exotic exchanges, like Synfutures, which do not have a fixed price tick.

*NOTE*: In a consolidated order book bid and ask often cross, which never happens in an order book of an individual exchange.

## Running the System

### Prerequisites

**Backend Dependencies:**
- Rust toolchain (install from https://rustup.rs/)
- Protocol Buffer Compiler (`protoc`)

**Installing protoc:**

On Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install -y protobuf-compiler
```

Verify installation:
```bash
protoc --version
```

**Frontend Dependencies:**
- Node.js (v18 or higher)
- pnpm (install with `npm install -g pnpm`)

### Backend (Rust)

Start the publisher which aggregates order book data and serves it via gRPC, WebSocket, and REST API:
```bash
cargo run --bin pricer
```

The publisher will:
- Connect to Binance and OKX WebSocket streams
- Aggregate order book data
- Serve gRPC on `127.0.0.1:50052` (for backward compatibility)
- Serve WebSocket on `ws://127.0.0.1:50051/ws`
- Serve REST API on `http://127.0.0.1:50051/api/*`

### Frontend (React)

1. Navigate to the frontend directory:
```bash
cd frontend
```

2. Install dependencies (if not already installed):
```bash
pnpm install
```

3. Start the development server (with Turbopack for faster builds):
```bash
pnpm run dev
```

The frontend will be available at `http://localhost:3000` and will automatically connect to the backend WebSocket server.

### Legacy Subscriber (Optional)

For testing the gRPC interface:
```bash
cargo run --bin consumer
```

The publisher should start first. The subscriber won't start otherwise.

Publishing will start after the first subscriber connects (it can serve multiple subscribers).

For simplicity, the consolidated order book will be published every 100ms via WebSocket (10 updates per second) and every second via gRPC.

The subscribers will log the received messages, the publisher will log a more detailed order book state (with separate volumes for Binance and OKX) every time it publishes.

## Difference with real production code
- WebSocket connection drops are not handled. Exchanges do drop connections periodically; in production, the market data component should reconnect once the connection is lost.
- Binance out of sync message - rare, but if it happens (message stream skips over some sequence numbers, the component should reconnect and rebuild the order book)
- I have hardcoded the gRPC server host and port for simplicity, symbols are also hardcoded for each exchange, price/size precisions are hardcoded
- For simplicity, I haven't dockerized the pricer and subscriber. It can be done if necessary, but in the case of a test assignment, since Rust has efficient package management, it may be excessive.
