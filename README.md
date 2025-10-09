
# Simple Price Aggregator in Rust

Aggregates spot ETHBTC prices from 2 sources:
- Binance
- OKX

## Assumptions and Reasoning
These sources are chosen due to the simplicity of the protocol and good liquidity. Calculating the entry price for a $ 50 million notional amount, as given in the formulation, is rarely feasible, even considering that these are one of the most liquid exchanges (Binance beats everything else by a margin). To make this choice, I used:
- rankings from https://coinmarketcap.com/rankings/exchanges/
- my own experience working with these exchanges

### Price/Volume Calculations
In my implementation, I decided to avoid working with floating point arithmetic due to a very bad experience I encountered when the protocol is designed to have price/size as floats. Potential problems: rounding error may accumualte while storing/manipulating float/double numbers; however, those floating-point numbers need to be converted to a fixed-precision floating-point string when communicating with the exchange. Such conversions may sometime throw price/size by a tick or so at unexpected moment, sometimes having a significant impact on P&L. Strategies where a single price tick off means a loss of money are specifically impacted (e.g., when your orders should aim to remain below the best price by 1 tick at most, without crossing, being repositioned each time price moves over a threshold).

It may be overkill for the task, but I decided to go for it to avoid unnecessary questions about rounding errors. A somewhat similar approach was used by one of my past employers, Tower Research Capital, when they represented price/size as pairs of integers instead of a single float each.

Also, not using integers instead of float/double avoids using epsilon-arithmetic in many places, making it more reliable to keep prices as map keys for storing bid/ask levels.

Prices/sizes are converted to floats at the consumer component side for display. Price precision chosen is 2, size precision chosen is 8. This is derived from current tick sizes and lot sizes in selected exchanges for BTCUSDT:

- Binance Reference Data
  - URL: https://api.binance.com/api/v3/exchangeInfo
  - ETHBTC lot size as of today:   0.0001  (size precision: 4)
  - ETHBTC tick size as of today: 0.00001  (price precision: 5)
- OKX Reference Data
  - URL: https://www.okx.com/api/v5/public/instruments?instType=SPOT
  - ETH-BTC lot size as of today: 0.000001 (size precision: 6)
  - ETH-BTC tick size as of today: 0.00001 (price precision: 5)

The chosen price/size precision is the maximum of the corresponding precisions on all 2 exchanges (6 for size precision and 5 for price precision). It would have been impossible to use this technique with more exotic exchanges, like Synfutures, which do not have a fixed price tick.

*NOTE*: In a consolidated order book bid and ask often cross, which never happens in an order book of an individual exchange.

## Building/Starting/Stopping
All to be done from the root folder of the project.
- Build: `docker-compose build`
- Start: `docker-compose up -d`
- Stopping: `docker-compose down`
- Tracing best bid/offer prices: `docker-compose logs -f get_bba`
- Tracing notional volume bands: `docker-compose logs -f get_vbd`
- Tracing mid price with "deviations": `docker-compose logs -f get_pbd`
- NOTE I did try uploading the image to DockerHub to save time on verifying the task, but considering the size of that image (several gigabytes), pulling it from DockerHub is not much faster, if at all, compared to building it from Dockerfile


## TODO
- WebSocket connection drop - implement reconnecting
- Binance out of sync meassage - implement re-fetching the snapshot
