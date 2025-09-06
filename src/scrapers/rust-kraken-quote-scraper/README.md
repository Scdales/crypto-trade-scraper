# Kraken Quote Scraper

This scraper connects to Kraken WebSocket API to collect XBT/USD ticker data.

## API Details

- **Endpoint**: `wss://ws.kraken.com`
- **Data**: Real-time ticker data for XBT/USD
- **Update Frequency**: Real-time

## Documentation Links

- [Kraken WebSocket API Documentation](https://docs.kraken.com/websockets/)
- [Ticker Channel Documentation](https://docs.kraken.com/websockets/#message-ticker)
- [Kraken API Reference](https://docs.kraken.com/rest/)

## Data Schema

```json
{
  "channel": "ticker",
  "type": "update",
  "data": [{
    "symbol": "XBT/USD",
    "ask": 42501.0,
    "ask_qty": 1.2345,
    "bid": 42500.0,
    "bid_qty": 2.3456,
    "change": 125.0,
    "change_pct": 0.295,
    "high": 43000.0,
    "last": 42500.5,
    "low": 42000.0,
    "volume": 1234.5678,
    "vwap": 42750.25
  }]
}
```

## Redis Keys

- `KRAKEN:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `KRAKEN:XBTUSD:QUOTE:BUY:VOL` - Best bid quantity
- `KRAKEN:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `KRAKEN:XBTUSD:QUOTE:SELL:VOL` - Best ask quantity

## Notes

- Subscription format: `{"method": "subscribe", "params": {"channel": "ticker", "symbol": ["XBT/USD"]}, "req_id": 1}`