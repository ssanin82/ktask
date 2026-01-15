import { OrderBookSnapshot } from '../App'
import './MetricsDashboard.css'

interface MetricsDashboardProps {
  snapshot: OrderBookSnapshot | null
}

const formatNumber = (value: number | null, decimals: number = 5): string => {
  if (value === null) return 'N/A'
  return value.toFixed(decimals)
}

const formatPercentage = (value: number | null): string => {
  if (value === null) return 'N/A'
  return `${(value * 100).toFixed(2)}%`
}

const MetricsDashboard: React.FC<MetricsDashboardProps> = ({ snapshot }) => {
  if (!snapshot) {
    return (
      <div className="metrics-dashboard">
        <h2>Metrics</h2>
        <div className="metrics-loading">Loading metrics...</div>
      </div>
    )
  }

  const midPrice = snapshot.mid_price ? snapshot.mid_price / 100000.0 : null
  const spread = snapshot.spread ? snapshot.spread / 100000.0 : null
  const spreadBps = midPrice && spread ? (spread / midPrice) * 10000 : null
  const vwapBid = snapshot.vwap_bid ? snapshot.vwap_bid / 100000.0 : null
  const vwapAsk = snapshot.vwap_ask ? snapshot.vwap_ask / 100000.0 : null

  return (
    <div className="metrics-dashboard">
      <h2>Real-Time Metrics</h2>
      
      <div className="metrics-grid">
        <div className="metric-card primary">
          <div className="metric-label">Mid Price</div>
          <div className="metric-value">{formatNumber(midPrice)}</div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Spread</div>
          <div className="metric-value spread-value">{formatNumber(spread)}</div>
          {spreadBps !== null && (
            <div className="metric-subvalue">{formatNumber(spreadBps, 2)} bps</div>
          )}
        </div>

        <div className="metric-card">
          <div className="metric-label">Best Bid</div>
          <div className="metric-value bid-value">
            {formatNumber(snapshot.best_bid ? snapshot.best_bid / 100000.0 : null)}
          </div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Best Ask</div>
          <div className="metric-value ask-value">
            {formatNumber(snapshot.best_ask ? snapshot.best_ask / 100000.0 : null)}
          </div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Bid Volume (Top 50)</div>
          <div className="metric-value">
            {formatNumber(snapshot.total_bid_volume / 1000000.0, 2)}
          </div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Ask Volume (Top 50)</div>
          <div className="metric-value">
            {formatNumber(snapshot.total_ask_volume / 1000000.0, 2)}
          </div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Order Book Imbalance</div>
          <div className="metric-value">
            {formatPercentage(snapshot.imbalance)}
          </div>
          {snapshot.imbalance !== null && (
            <div className="imbalance-bar">
              <div
                className="imbalance-fill"
                style={{
                  width: `${snapshot.imbalance * 100}%`,
                  background: snapshot.imbalance > 0.5
                    ? 'linear-gradient(to right, #4caf50, #81c784)'
                    : 'linear-gradient(to right, #f44336, #e57373)',
                }}
              />
            </div>
          )}
        </div>

        <div className="metric-card">
          <div className="metric-label">VWAP Bid</div>
          <div className="metric-value">{formatNumber(vwapBid)}</div>
        </div>

        <div className="metric-card">
          <div className="metric-label">VWAP Ask</div>
          <div className="metric-value">{formatNumber(vwapAsk)}</div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Depth Bid (5 bps)</div>
          <div className="metric-value">
            {formatNumber(snapshot.depth_bid_5bps / 1000000.0, 2)}
          </div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Depth Ask (5 bps)</div>
          <div className="metric-value">
            {formatNumber(snapshot.depth_ask_5bps / 1000000.0, 2)}
          </div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Depth Bid (10 bps)</div>
          <div className="metric-value">
            {formatNumber(snapshot.depth_bid_10bps / 1000000.0, 2)}
          </div>
        </div>

        <div className="metric-card">
          <div className="metric-label">Depth Ask (10 bps)</div>
          <div className="metric-value">
            {formatNumber(snapshot.depth_ask_10bps / 1000000.0, 2)}
          </div>
        </div>
      </div>

      <div className="update-counts">
        <h3>Update Counts</h3>
        <div className="update-counts-grid">
          <div className="update-count-item">
            <span className="exchange-name binance">Binance</span>
            <span className="update-count">{snapshot.update_counts['BINANCE'] || 0}</span>
          </div>
          <div className="update-count-item">
            <span className="exchange-name okx">OKX</span>
            <span className="update-count">{snapshot.update_counts['OKX'] || 0}</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default MetricsDashboard
