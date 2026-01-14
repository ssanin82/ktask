import { useMemo } from 'react'
import { OrderBookSnapshot } from '../App'
import './ExchangeComparison.css'

interface ExchangeComparisonProps {
  snapshot: OrderBookSnapshot | null
}

const ExchangeComparison: React.FC<ExchangeComparisonProps> = ({ snapshot }) => {
  const { binanceStats, okxStats } = useMemo(() => {
    if (!snapshot) {
      return { binanceStats: null, okxStats: null }
    }

    let binanceBidVolume = 0
    let binanceAskVolume = 0
    let okxBidVolume = 0
    let okxAskVolume = 0
    let binanceLevels = 0
    let okxLevels = 0

    snapshot.bids.forEach(bid => {
      const binanceSize = bid.by_source['BINANCE'] || 0
      const okxSize = bid.by_source['OKX'] || 0
      if (binanceSize > 0) {
        binanceBidVolume += binanceSize
        binanceLevels++
      }
      if (okxSize > 0) {
        okxBidVolume += okxSize
        okxLevels++
      }
    })

    snapshot.asks.forEach(ask => {
      const binanceSize = ask.by_source['BINANCE'] || 0
      const okxSize = ask.by_source['OKX'] || 0
      if (binanceSize > 0) {
        binanceAskVolume += binanceSize
        binanceLevels++
      }
      if (okxSize > 0) {
        okxAskVolume += okxSize
        okxLevels++
      }
    })

    const binanceTotal = binanceBidVolume + binanceAskVolume
    const okxTotal = okxBidVolume + okxAskVolume
    const grandTotal = binanceTotal + okxTotal

    return {
      binanceStats: {
        bidVolume: binanceBidVolume / 1000000.0,
        askVolume: binanceAskVolume / 1000000.0,
        totalVolume: binanceTotal / 1000000.0,
        levels: binanceLevels,
        marketShare: grandTotal > 0 ? (binanceTotal / grandTotal) * 100 : 0,
        updateCount: snapshot.update_counts['BINANCE'] || 0,
      },
      okxStats: {
        bidVolume: okxBidVolume / 1000000.0,
        askVolume: okxAskVolume / 1000000.0,
        totalVolume: okxTotal / 1000000.0,
        levels: okxLevels,
        marketShare: grandTotal > 0 ? (okxTotal / grandTotal) * 100 : 0,
        updateCount: snapshot.update_counts['OKX'] || 0,
      },
    }
  }, [snapshot])

  if (!snapshot || !binanceStats || !okxStats) {
    return (
      <div className="exchange-comparison">
        <h2>Exchange Comparison</h2>
        <div className="comparison-loading">Loading comparison data...</div>
      </div>
    )
  }

  return (
    <div className="exchange-comparison">
      <h2>Exchange Comparison</h2>
      
      <div className="exchange-cards">
        <div className="exchange-card binance-card">
          <div className="exchange-header">
            <h3 className="exchange-name binance">Binance</h3>
            <div className="exchange-market-share">
              {binanceStats.marketShare.toFixed(1)}% Market Share
            </div>
          </div>
          
          <div className="exchange-metrics">
            <div className="exchange-metric">
              <span className="metric-label">Bid Volume</span>
              <span className="metric-value bid">{binanceStats.bidVolume.toFixed(2)}</span>
            </div>
            <div className="exchange-metric">
              <span className="metric-label">Ask Volume</span>
              <span className="metric-value ask">{binanceStats.askVolume.toFixed(2)}</span>
            </div>
            <div className="exchange-metric">
              <span className="metric-label">Total Volume</span>
              <span className="metric-value">{binanceStats.totalVolume.toFixed(2)}</span>
            </div>
            <div className="exchange-metric">
              <span className="metric-label">Active Levels</span>
              <span className="metric-value">{binanceStats.levels}</span>
            </div>
            <div className="exchange-metric">
              <span className="metric-label">Update Count</span>
              <span className="metric-value">{binanceStats.updateCount}</span>
            </div>
          </div>

          <div className="market-share-bar">
            <div
              className="market-share-fill binance-fill"
              style={{ width: `${binanceStats.marketShare}%` }}
            />
          </div>
        </div>

        <div className="exchange-card okx-card">
          <div className="exchange-header">
            <h3 className="exchange-name okx">OKX</h3>
            <div className="exchange-market-share">
              {okxStats.marketShare.toFixed(1)}% Market Share
            </div>
          </div>
          
          <div className="exchange-metrics">
            <div className="exchange-metric">
              <span className="metric-label">Bid Volume</span>
              <span className="metric-value bid">{okxStats.bidVolume.toFixed(2)}</span>
            </div>
            <div className="exchange-metric">
              <span className="metric-label">Ask Volume</span>
              <span className="metric-value ask">{okxStats.askVolume.toFixed(2)}</span>
            </div>
            <div className="exchange-metric">
              <span className="metric-label">Total Volume</span>
              <span className="metric-value">{okxStats.totalVolume.toFixed(2)}</span>
            </div>
            <div className="exchange-metric">
              <span className="metric-label">Active Levels</span>
              <span className="metric-value">{okxStats.levels}</span>
            </div>
            <div className="exchange-metric">
              <span className="metric-label">Update Count</span>
              <span className="metric-value">{okxStats.updateCount}</span>
            </div>
          </div>

          <div className="market-share-bar">
            <div
              className="market-share-fill okx-fill"
              style={{ width: `${okxStats.marketShare}%` }}
            />
          </div>
        </div>
      </div>
    </div>
  )
}

export default ExchangeComparison
