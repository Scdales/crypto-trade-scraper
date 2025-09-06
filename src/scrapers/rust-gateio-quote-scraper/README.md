# Gate.io Quote Scraper

This scraper connects to Gate.io WebSocket API to collect BTC_USDT ticker data.

## API Details

- **Endpoint**: `wss://api.gateio.ws/ws/v4/`
- **Data**: Real-time ticker data for BTC_USDT
- **Update Frequency**: Real-time

## Documentation Links

- [Gate.io WebSocket API v4 Documentation](https://www.gate.io/docs/developers/apiv4/ws/en/)
- [Ticker Channel Documentation](https://www.gate.io/docs/developers/apiv4/ws/en/#tickers-channel)
- [Gate.io API Reference](https://www.gate.io/docs/developers/apiv4/en/)

## Data Schema

```json
{
  "time": 1704276930,
  "time_ms": 1704276930123,
  "channel": "spot.tickers",
  "event": "update",
  "result": {
    "currency_pair": "BTC_USDT",
    "last": "42500.00",
    "lowest_ask": "42501.00",
    "highest_bid": "42500.00",
    "change_percentage": "1.23",
    "base_volume": "1234.5678",
    "quote_volume": "52345678.90",
    "high_24h": "43000.00",
    "low_24h": "42000.00"
  }
}
```

## Redis Keys

- `GATEIO:XBTUSD:QUOTE:BUY:PRICE` - Highest bid price
- `GATEIO:XBTUSD:QUOTE:BUY:VOL` - Base volume / 2 (estimated bid volume)
- `GATEIO:XBTUSD:QUOTE:SELL:PRICE` - Lowest ask price
- `GATEIO:XBTUSD:QUOTE:SELL:VOL` - Base volume / 2 (estimated ask volume)

## Notes

- Subscription format: `{"time": timestamp, "channel": "spot.tickers", "event": "subscribe", "payload": ["BTC_USDT"]}`