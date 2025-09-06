# OKX Quote Scraper

This scraper connects to OKX WebSocket API to collect BTC-USDT ticker data.

## API Details

- **Endpoint**: `wss://ws.okx.com:8443/ws/v5/public`
- **Data**: Real-time ticker data for BTC-USDT
- **Update Frequency**: Real-time

## Documentation Links

- [OKX WebSocket API v5 Documentation](https://www.okx.com/docs-v5/en/#websocket-api-public-channels-tickers-channel)
- [OKX API Overview](https://www.okx.com/docs-v5/en/#overview)
- [Ticker Channel Documentation](https://www.okx.com/docs-v5/en/#websocket-api-public-channels-tickers-channel)

## Data Schema

```json
{
  "arg": {
    "channel": "tickers",
    "instId": "BTC-USDT"
  },
  "data": [{
    "instId": "BTC-USDT",
    "last": "42500.00",
    "lastSz": "0.1234",
    "askPx": "42501.00",
    "askSz": "1.5678",
    "bidPx": "42500.00",
    "bidSz": "2.3456",
    "ts": "1704276930123"
  }]
}
```

## Redis Keys

- `OKX:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `OKX:XBTUSD:QUOTE:BUY:VOL` - Best bid volume
- `OKX:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `OKX:XBTUSD:QUOTE:SELL:VOL` - Best ask volume

## Notes

- Subscription format: `{"op": "subscribe", "args": [{"channel": "tickers", "instId": "BTC-USDT"}]}`