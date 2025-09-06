# Bybit Quote Scraper

This scraper connects to Bybit's WebSocket API to collect BTC/USDT ticker data.

## API Details

- **Endpoint**: `wss://stream.bybit.com/v5/public/spot`
- **Data**: 24h ticker statistics for BTC/USDT spot trading
- **Update Frequency**: Real-time

## Documentation Links

- [Bybit WebSocket API v5 Documentation](https://bybit-exchange.github.io/docs/v5/websocket/public/ticker)
- [Bybit API Overview](https://bybit-exchange.github.io/docs/v5/intro)
- [Ticker Stream Documentation](https://bybit-exchange.github.io/docs/v5/websocket/public/ticker)

## Data Schema

```json
{
  "topic": "tickers.BTCUSDT",
  "type": "snapshot",
  "data": {
    "symbol": "BTCUSDT",
    "lastPrice": "42500.00",
    "highPrice24h": "43000.00",
    "lowPrice24h": "42000.00",
    "prevPrice24h": "42250.00",
    "volume24h": "1234.5678",
    "turnover24h": "52345678.90",
    "price24hPcnt": "0.0059",
    "usdIndexPrice": "42501.23"
  }
}
```

## Redis Keys

- `BYBIT:XBTUSD:QUOTE:BUY:PRICE` - Last trade price (used as bid)
- `BYBIT:XBTUSD:QUOTE:BUY:VOL` - 24h volume / 2
- `BYBIT:XBTUSD:QUOTE:SELL:PRICE` - Last trade price (used as ask)
- `BYBIT:XBTUSD:QUOTE:SELL:VOL` - 24h volume / 2