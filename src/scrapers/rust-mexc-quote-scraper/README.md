# MEXC Quote Scraper

This scraper connects to MEXC WebSocket API to collect BTC/USDT ticker data.

## API Details

- **Endpoint**: `wss://wbs.mexc.com/ws`
- **Data**: Real-time book ticker data for BTCUSDT
- **Update Frequency**: Real-time

## Documentation Links

- [MEXC WebSocket API Documentation](https://mexcdevelop.github.io/apidocs/spot_v3_en/#websocket-market-streams)
- [Book Ticker Stream](https://mexcdevelop.github.io/apidocs/spot_v3_en/#individual-symbol-book-ticker-streams)
- [MEXC API Reference](https://mexcdevelop.github.io/apidocs/spot_v3_en/)

## Data Schema

```json
{
  "stream": "btcusdt@bookTicker",
  "data": {
    "lastUpdateId": 1027024,
    "symbol": "BTCUSDT",
    "bidPrice": "42500.00",
    "bidQty": "1.2345",
    "askPrice": "42501.00",
    "askQty": "2.3456"
  }
}
```

## Redis Keys

- `MEXC:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `MEXC:XBTUSD:QUOTE:BUY:VOL` - Best bid quantity
- `MEXC:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `MEXC:XBTUSD:QUOTE:SELL:VOL` - Best ask quantity

## Notes

- Subscription format: `{"method": "SUBSCRIPTION", "params": ["btcusdt@bookTicker"], "id": 1}`