# Bitfinex Quote Scraper

This scraper connects to Bitfinex WebSocket API to collect BTCUSD ticker data.

## API Details

- **Endpoint**: `wss://api-pub.bitfinex.com/ws/2`
- **Data**: Real-time ticker data for BTCUSD
- **Update Frequency**: Real-time

## Documentation Links

- [Bitfinex WebSocket API v2 Documentation](https://docs.bitfinex.com/reference/ws-public-ticker)
- [Bitfinex API Overview](https://docs.bitfinex.com/docs)
- [Ticker Channel Documentation](https://docs.bitfinex.com/reference/ws-public-ticker)

## Data Schema

```json
[
  0,           // CHANNEL_ID
  [
    42500.0,   // BID
    1.2345,    // BID_SIZE
    42501.0,   // ASK
    2.3456,    // ASK_SIZE
    125.0,     // DAILY_CHANGE
    0.295,     // DAILY_CHANGE_RELATIVE
    42500.5,   // LAST_PRICE
    1234.5678, // VOLUME
    43000.0,   // HIGH
    42000.0    // LOW
  ]
]
```

## Redis Keys

- `BITFINEX:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `BITFINEX:XBTUSD:QUOTE:BUY:VOL` - Best bid size
- `BITFINEX:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `BITFINEX:XBTUSD:QUOTE:SELL:VOL` - Best ask size

## Notes

- Subscription format: `{"event": "subscribe", "channel": "ticker", "symbol": "BTCUSD"}`
- Data arrives as array format, not JSON objects
- Channel ID mapping required for message routing