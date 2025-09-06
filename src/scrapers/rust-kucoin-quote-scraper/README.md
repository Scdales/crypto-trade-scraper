# KuCoin Quote Scraper

This scraper connects to KuCoin's WebSocket API to collect BTC-USDT ticker data.

## API Details

- **Token Endpoint**: `https://api.kucoin.com/api/v1/bullet-public`
- **WebSocket**: Dynamic endpoint provided by token response
- **Data**: Real-time ticker data for BTC-USDT
- **Update Frequency**: Real-time

## Documentation Links

- [KuCoin WebSocket API Documentation](https://www.kucoin.com/docs/websocket/introduction)
- [Public Token Documentation](https://www.kucoin.com/docs/websocket/basic-info/apply-connect-token/public-token-no-authentication-required-)
- [Ticker Subscription](https://www.kucoin.com/docs/websocket/basic-info/subscribe/introduction)
- [KuCoin API Reference](https://www.kucoin.com/docs/rest/introduction)

## Data Schema

```json
{
  "type": "message",
  "topic": "/market/ticker:BTC-USDT",
  "subject": "trade.ticker",
  "data": {
    "sequence": "1545896669145",
    "price": "42500.00",
    "size": "0.1234",
    "bestAsk": "42501.00",
    "bestAskSize": "1.5678",
    "bestBid": "42500.00",
    "bestBidSize": "2.3456"
  }
}
```

## Redis Keys

- `KUCOIN:XBTUSD:QUOTE:BUY:PRICE` - Best bid price
- `KUCOIN:XBTUSD:QUOTE:BUY:VOL` - Best bid volume
- `KUCOIN:XBTUSD:QUOTE:SELL:PRICE` - Best ask price
- `KUCOIN:XBTUSD:QUOTE:SELL:VOL` - Best ask volume

## Notes

- Requires token-based authentication for WebSocket connection
- Dynamic WebSocket endpoint URL provided by bullet-public API
- Uses server-provided ping interval for connection keepalive
- Subscription format: `{"id": timestamp, "type": "subscribe", "topic": "/market/ticker:BTC-USDT", "response": true}`