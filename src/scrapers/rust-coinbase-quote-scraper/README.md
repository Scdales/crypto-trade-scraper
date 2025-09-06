# Coinbase Quote Scraper

This scraper connects to Coinbase Advanced Trade WebSocket API to collect BTC-USD ticker data.

## API Details

- **Endpoint**: `wss://advanced-trade-ws.coinbase.com`
- **Data**: Real-time ticker updates for BTC-USD
- **Update Frequency**: Real-time

## Documentation Links

- [Coinbase Advanced Trade WebSocket API](https://docs.cloud.coinbase.com/advanced-trade-api/docs/ws-overview)
- [Ticker Channel Documentation](https://docs.cloud.coinbase.com/advanced-trade-api/docs/ws-channels#ticker-channel)
- [Coinbase API Documentation](https://docs.cloud.coinbase.com/advanced-trade-api/docs/welcome)

## Data Schema

```json
{
  "channel": "ticker",
  "client_id": "",
  "timestamp": "2024-01-03T10:15:30.123456Z",
  "sequence_num": 0,
  "events": [{
    "type": "update",
    "tickers": [{
      "type": "ticker",
      "product_id": "BTC-USD",
      "price": "42500.00",
      "volume_24_h": "1234.56789",
      "low_24_h": "42000.00",
      "high_24_h": "43000.00",
      "low_52_w": "15000.00",
      "high_52_w": "69000.00",
      "price_percent_chg_24_h": "1.23"
    }]
  }]
}
```

## Redis Keys

- `COINBASE:XBTUSD:QUOTE:BUY:PRICE` - Current price (used as bid)
- `COINBASE:XBTUSD:QUOTE:BUY:VOL` - 24h volume / 2
- `COINBASE:XBTUSD:QUOTE:SELL:PRICE` - Current price (used as ask)
- `COINBASE:XBTUSD:QUOTE:SELL:VOL` - 24h volume / 2