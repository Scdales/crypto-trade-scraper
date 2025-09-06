# Bitget Quote Scraper

This scraper connects to Bitget WebSocket API to collect BTC/USDT ticker data.

## API Details

- **Endpoint**: `wss://ws.bitget.com/mix/v1/stream`
- **Data**: Real-time ticker data for BTCUSDT
- **Update Frequency**: Real-time

## Documentation Links

- [Bitget WebSocket API Documentation](https://www.bitget.com/api-doc/spot/websocket/public/Tickers-Channel)
- [Bitget API Overview](https://www.bitget.com/api-doc/common/intro)
- [Ticker Channel Documentation](https://www.bitget.com/api-doc/spot/websocket/public/Tickers-Channel)

## Data Schema

```json
{
  "action": "snapshot",
  "arg": {
    "instType": "sp",
    "channel": "ticker",
    "instId": "BTCUSDT"
  },
  "data": [{
    "instId": "BTCUSDT",
    "last": "42500.00",
    "open24h": "42250.00",
    "high24h": "43000.00",
    "low24h": "42000.00",
    "bestBid": "42500.00",
    "bestAsk": "42501.00",
    "baseVolume": "1234.5678",
    "quoteVolume": "52345678.90",
    "ts": "1704276930123",
    "launchTime": "1640995200000"
  }]
}
```

## Redis Keys

- `BITGET:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `BITGET:XBTUSD:QUOTE:BUY:VOL` - Base volume / 2 (estimated bid volume)
- `BITGET:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `BITGET:XBTUSD:QUOTE:SELL:VOL` - Base volume / 2 (estimated ask volume)

## Notes

- Subscription format: `{"op": "subscribe", "args": [{"instType": "sp", "channel": "ticker", "instId": "BTCUSDT"}]}`