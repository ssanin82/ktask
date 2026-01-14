import { useState, useEffect } from 'react'
import OrderBookLadder from './components/OrderBookLadder'
import MetricsDashboard from './components/MetricsDashboard'
import TimeSeriesChart from './components/TimeSeriesChart'
import ExchangeComparison from './components/ExchangeComparison'
import './App.css'

export interface OrderBookSnapshot {
  timestamp: string
  spread: number | null
  best_bid: number | null
  best_ask: number | null
  mid_price: number | null
  bids: Array<{
    price: number
    total: number
    by_source: Record<string, number>
  }>
  asks: Array<{
    price: number
    total: number
    by_source: Record<string, number>
  }>
  total_bid_volume: number
  total_ask_volume: number
  update_counts: Record<string, number>
  vwap_bid: number | null
  vwap_ask: number | null
  depth_bid_5bps: number
  depth_ask_5bps: number
  depth_bid_10bps: number
  depth_ask_10bps: number
  imbalance: number | null
}

function App() {
  const [snapshot, setSnapshot] = useState<OrderBookSnapshot | null>(null)
  const [history, setHistory] = useState<Array<{ time: string; spread: number | null; midPrice: number | null }>>([])
  const [connected, setConnected] = useState(false)

  useEffect(() => {
    const ws = new WebSocket('ws://127.0.0.1:3001/ws')

    ws.onopen = () => {
      console.log('WebSocket connected')
      setConnected(true)
    }

    ws.onmessage = (event) => {
      try {
        const data: OrderBookSnapshot = JSON.parse(event.data)
        setSnapshot(data)
        
        // Update history for charts (keep last 100 points)
        setHistory(prev => {
          const newHistory = [...prev, {
            time: new Date(data.timestamp).toLocaleTimeString(),
            spread: data.spread,
            midPrice: data.mid_price
          }]
          return newHistory.slice(-100)
        })
      } catch (error) {
        console.error('Error parsing WebSocket message:', error)
      }
    }

    ws.onerror = (error) => {
      console.error('WebSocket error:', error)
      setConnected(false)
    }

    ws.onclose = () => {
      console.log('WebSocket disconnected')
      setConnected(false)
      // Attempt to reconnect after 3 seconds
      setTimeout(() => {
        window.location.reload()
      }, 3000)
    }

    return () => {
      ws.close()
    }
  }, [])

  return (
    <div className="app">
      <header className="app-header">
        <h1>ETH/BTC Order Book - HFT Dashboard</h1>
        <div className={`connection-status ${connected ? 'connected' : 'disconnected'}`}>
          <span className="status-dot"></span>
          {connected ? 'Connected' : 'Disconnected'}
        </div>
      </header>

      <div className="dashboard-grid">
        <div className="metrics-panel">
          <MetricsDashboard snapshot={snapshot} />
        </div>

        <div className="orderbook-panel">
          <OrderBookLadder snapshot={snapshot} />
        </div>

        <div className="charts-panel">
          <TimeSeriesChart history={history} />
        </div>

        <div className="exchange-panel">
          <ExchangeComparison snapshot={snapshot} />
        </div>
      </div>
    </div>
  )
}

export default App
