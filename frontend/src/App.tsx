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
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let ws: WebSocket | null = null
    let reconnectTimeout: NodeJS.Timeout | null = null

    const connect = () => {
      try {
        ws = new WebSocket('ws://127.0.0.1:50051/ws')

        ws.onopen = () => {
          console.log('WebSocket connected')
          setConnected(true)
          setError(null)
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
            setError('Failed to parse data from server')
          }
        }

        ws.onerror = (error) => {
          console.error('WebSocket error:', error)
          setConnected(false)
          setError('WebSocket connection error. Make sure the backend is running on port 50051.')
        }

        ws.onclose = (event) => {
          console.log('WebSocket disconnected', event.code, event.reason)
          setConnected(false)
          if (event.code !== 1000) {
            setError('Connection lost. Attempting to reconnect...')
          }
          // Attempt to reconnect after 3 seconds
          reconnectTimeout = setTimeout(() => {
            connect()
          }, 3000)
        }
      } catch (error) {
        console.error('Failed to create WebSocket:', error)
        setError('Failed to connect to backend. Make sure the backend is running on port 50051.')
        setConnected(false)
      }
    }

    connect()

    return () => {
      if (reconnectTimeout) {
        clearTimeout(reconnectTimeout)
      }
      if (ws) {
        ws.close()
      }
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

      {error && (
        <div className="error-banner">
          <strong>Error:</strong> {error}
        </div>
      )}

      {!connected && !snapshot && (
        <div className="loading-message">
          <p>Connecting to backend...</p>
          <p style={{ fontSize: '14px', color: 'rgba(255, 255, 255, 0.6)', marginTop: '10px' }}>
            Make sure the backend is running: <code>cargo run --bin pricer</code>
          </p>
        </div>
      )}

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
