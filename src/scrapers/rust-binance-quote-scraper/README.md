# Binance Quote Scraper

This scraper connects to Binance's WebSocket API to collect BTC/USDT ticker data.

## API Details

- **Endpoint**: `wss://stream.binance.com:9443/ws/btcusdt@bookTicker`
- **Data**: Best bid/ask prices and volumes for BTC/USDT
- **Update Frequency**: Real-time

## Documentation Links

- [Binance WebSocket API Documentation](https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams)
- [Book Ticker Stream](https://binance-docs.github.io/apidocs/spot/en/#individual-symbol-book-ticker-streams)
- [Binance API GitHub](https://github.com/binance/binance-spot-api-docs)

## Data Schema

```json
{
  "u": 400900217,     // order book updateId
  "s": "BNBUSDT",     // symbol
  "b": "25.35190000", // best bid price
  "B": "31.21000000", // best bid qty
  "a": "25.36520000", // best ask price
  "A": "40.66000000"  // best ask qty
}
```

## Redis Keys

- `BINANCE:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `BINANCE:XBTUSD:QUOTE:BUY:VOL` - Best bid volume
- `BINANCE:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `BINANCE:XBTUSD:QUOTE:SELL:VOL` - Best ask volume