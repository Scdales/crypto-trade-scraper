# BitMEX Quote Scraper

This scraper connects to BitMEX's WebSocket API to collect XBTUSD quote data.

## API Details

- **Endpoint**: `wss://ws.bitmex.com/realtime?subscribe=quote:XBTUSD`
- **Data**: Best bid/ask prices and volumes for XBTUSD perpetual contract
- **Update Frequency**: Real-time

## Documentation Links

- [BitMEX WebSocket API Documentation](https://www.bitmex.com/app/wsAPI)
- [BitMEX API Explorer](https://www.bitmex.com/api/explorer/)
- [Quote Data Schema](https://www.bitmex.com/app/wsAPI#Quote)

## Data Schema

```json
{
  "table": "quote",
  "action": "partial|insert|update|delete",
  "data": [{
    "timestamp": "2024-01-03T00:09:50.444Z",
    "symbol": "XBTUSD",
    "bidPrice": 42500.5,
    "bidSize": 1000,
    "askPrice": 42501.0,
    "askSize": 500
  }]
}
```

## Redis Keys

- `BITMEX:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `BITMEX:XBTUSD:QUOTE:BUY:VOL` - Best bid volume
- `BITMEX:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `BITMEX:XBTUSD:QUOTE:SELL:VOL` - Best ask volume