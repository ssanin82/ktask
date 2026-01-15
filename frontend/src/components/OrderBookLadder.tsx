import { useMemo } from 'react'
import { OrderBookSnapshot } from '../App'
import './OrderBookLadder.css'

interface OrderBookLadderProps {
  snapshot: OrderBookSnapshot | null
}

const formatPrice = (price: number | null, precision: number = 5): string => {
  if (price === null || isNaN(price)) return 'N/A'
  return price.toFixed(precision)
}

const formatSize = (size: number, precision: number = 6): string => {
  if (isNaN(size)) return '0.000000'
  return size.toFixed(precision)
}

const OrderBookLadder: React.FC<OrderBookLadderProps> = ({ snapshot }) => {
  const { bids, asks, maxVolume } = useMemo(() => {
    if (!snapshot) {
      return { bids: [], asks: [], maxVolume: 0 }
    }

    // Only show top 5 levels per side
    const topBids = snapshot.bids.slice(0, 5)
    const topAsks = snapshot.asks.slice(0, 5)

    const processedBids = topBids.map(b => ({
      ...b,
      priceFloat: b.price / 100000.0,
      sizeFloat: b.total / 1000000.0,
      binanceSize: (b.by_source['BINANCE'] || 0) / 1000000.0,
      okxSize: (b.by_source['OKX'] || 0) / 1000000.0,
    }))

    const processedAsks = topAsks.map(a => ({
      ...a,
      priceFloat: a.price / 100000.0,
      sizeFloat: a.total / 1000000.0,
      binanceSize: (a.by_source['BINANCE'] || 0) / 1000000.0,
      okxSize: (a.by_source['OKX'] || 0) / 1000000.0,
    }))

    const allVolumes = [...processedBids, ...processedAsks].map(x => x.sizeFloat)
    const maxVolume = Math.max(...allVolumes, 1)

    return {
      bids: processedBids.reverse(), // Highest first
      asks: processedAsks, // Lowest first
      maxVolume,
    }
  }, [snapshot])

  const midPrice = snapshot?.mid_price ? snapshot.mid_price / 100000.0 : null
  const spread = snapshot?.spread ? snapshot.spread / 100000.0 : null

  if (!snapshot) {
    return (
      <div className="orderbook-ladder">
        <div className="orderbook-header">
          <h2>Order Book</h2>
        </div>
        <div className="orderbook-loading">Loading order book data...</div>
      </div>
    )
  }

  return (
    <div className="orderbook-ladder">
      <div className="orderbook-header">
        <h2>Order Book Ladder</h2>
        <div className="orderbook-info">
          {midPrice !== null && (
            <span className="mid-price">Mid: {formatPrice(midPrice)}</span>
          )}
          {spread !== null && (
            <span className="spread">Spread: {formatPrice(spread)}</span>
          )}
        </div>
      </div>

      <div className="orderbook-container">
        <div className="orderbook-side asks-side">
          <div className="orderbook-side-header">
            <span>Price</span>
            <span>Size</span>
            <span>Total</span>
            <span>Binance</span>
            <span>OKX</span>
          </div>
          {asks.map((ask, idx) => {
            const widthPercent = (ask.sizeFloat / maxVolume) * 100
            const isBestAsk = idx === 0
            return (
              <div
                key={`ask-${ask.price}`}
                className={`orderbook-row ask-row ${isBestAsk ? 'best-level' : ''}`}
                style={{
                  background: `linear-gradient(to left, rgba(244, 67, 54, ${0.1 + widthPercent / 200}), transparent)`,
                }}
              >
                <span className="price ask-price">{formatPrice(ask.priceFloat)}</span>
                <span className="size">{formatSize(ask.sizeFloat)}</span>
                <span className="total">{formatSize(ask.sizeFloat)}</span>
                <span className="exchange binance">{formatSize(ask.binanceSize)}</span>
                <span className="exchange okx">{formatSize(ask.okxSize)}</span>
                <div
                  className="volume-bar ask-bar"
                  style={{ width: `${widthPercent}%` }}
                />
              </div>
            )
          })}
        </div>

        <div className="orderbook-divider">
          <div className="mid-price-line">
            {midPrice !== null && (
              <span className="mid-price-label">{formatPrice(midPrice)}</span>
            )}
          </div>
        </div>

        <div className="orderbook-side bids-side">
          <div className="orderbook-side-header">
            <span>Price</span>
            <span>Size</span>
            <span>Total</span>
            <span>Binance</span>
            <span>OKX</span>
          </div>
          {bids.map((bid, idx) => {
            const widthPercent = (bid.sizeFloat / maxVolume) * 100
            const isBestBid = idx === 0
            return (
              <div
                key={`bid-${bid.price}`}
                className={`orderbook-row bid-row ${isBestBid ? 'best-level' : ''}`}
                style={{
                  background: `linear-gradient(to left, rgba(76, 175, 80, ${0.1 + widthPercent / 200}), transparent)`,
                }}
              >
                <span className="price bid-price">{formatPrice(bid.priceFloat)}</span>
                <span className="size">{formatSize(bid.sizeFloat)}</span>
                <span className="total">{formatSize(bid.sizeFloat)}</span>
                <span className="exchange binance">{formatSize(bid.binanceSize)}</span>
                <span className="exchange okx">{formatSize(bid.okxSize)}</span>
                <div
                  className="volume-bar bid-bar"
                  style={{ width: `${widthPercent}%` }}
                />
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}

export default OrderBookLadder
