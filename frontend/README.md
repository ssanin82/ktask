# Order Book Visualization Frontend

A professional HFT-style order book visualization dashboard built with React and TypeScript.

## Features

- **Real-time Order Book Ladder**: Classic HFT-style visualization with bids and asks
- **Metrics Dashboard**: Comprehensive real-time metrics including spread, VWAP, depth analysis
- **Time-Series Charts**: Historical tracking of mid price and spread
- **Exchange Comparison**: Side-by-side comparison of Binance and OKX contributions
- **Dark Theme**: Professional dark theme optimized for trading environments

## Setup

1. Install dependencies:
```bash
npm install
```

2. Start the development server:
```bash
npm run dev
```

The frontend will be available at `http://localhost:3000`

## Prerequisites

- Node.js and npm installed
- The Rust backend server must be running on `http://127.0.0.1:3001`

## Build for Production

```bash
npm run build
```

The built files will be in the `dist` directory.
