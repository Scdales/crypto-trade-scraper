# HTX Quote Scraper

This scraper connects to HTX (formerly Huobi) WebSocket API to collect BTC/USDT ticker data.

## API Details

- **Endpoint**: `wss://api.huobi.pro/ws`
- **Data**: Real-time ticker data for BTC/USDT
- **Update Frequency**: Real-time
- **Compression**: GZIP compressed messages

## Documentation Links

- [HTX WebSocket API Documentation](https://huobiapi.github.io/docs/spot/v1/en/#websocket-market-data)
- [Market Ticker Documentation](https://huobiapi.github.io/docs/spot/v1/en/#market-ticker)
- [HTX API Reference](https://www.htx.com/en-us/opend/newApiPages/)

## Data Schema

```json
{
  "ch": "market.btcusdt.ticker",
  "ts": 1704276930000,
  "tick": {
    "amount": 1234.567,
    "ask": 42501.0,
    "bid": 42500.0,
    "close": 42500.5,
    "count": 12345,
    "high": 43000.0,
    "low": 42000.0,
    "open": 42250.0,
    "vol": 52345.678
  }
}
```

## Redis Keys

- `HTX:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `HTX:XBTUSD:QUOTE:BUY:VOL` - Volume / 2 (estimated bid volume)
- `HTX:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `HTX:XBTUSD:QUOTE:SELL:VOL` - Volume / 2 (estimated ask volume)

## Notes

- Messages are GZIP compressed and require decompression
- HTX uses ping/pong mechanism for connection keepalive
- Subscription format: `{"sub": "market.btcusdt.ticker", "id": "id1"}`