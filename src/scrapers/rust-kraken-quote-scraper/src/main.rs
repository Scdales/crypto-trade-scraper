use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tungstenite::{connect, Message};
use url::Url;
use redis::{Connection, RedisError};
use redis_ts::{TsCommands, TsOptions, TsDuplicatePolicy};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct KrakenTickerData {
    symbol: String,
    bid: f64,
    bid_qty: f64,
    ask: f64,
    ask_qty: f64,
    last: f64,
    volume: f64,
    vwap: f64,
    low: f64,
    high: f64,
    change: f64,
    change_pct: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct KrakenTickerMessage {
    channel: String,
    #[serde(rename = "type")]
    msg_type: Option<String>,
    #[serde(default)]
    data: Vec<KrakenTickerData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct KrakenSubscriptionParams {
    channel: String,
    symbol: Vec<String>,
    event_trigger: String,
    snapshot: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct KrakenSubscriptionMessage {
    method: String,
    params: KrakenSubscriptionParams,
}

const KEY_PREFIX: &str = "KRAKEN:XBTUSD:QUOTE";
const KRAKEN_WS_API: &str = "wss://ws.kraken.com/v2";
const RETENTION_TIME: u64 = 3600000;

fn add_current_data(con: &mut Connection, ts: u64, ticker: &KrakenTickerData, options: &TsOptions) {
    let bid_price = ticker.bid;
    let bid_vol = ticker.bid_qty;
    let ask_price = ticker.ask;
    let ask_vol = ticker.ask_qty;


    let buy_options = options.clone().label("SIDE", "BUY").label("SUB", "QUOTE");
    let buy_price_key = format!("{}:BUY:PRICE", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(buy_price_key, ts, bid_price, buy_options.clone().label("GROUP", "PRICE"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding buy price to redis: {}", print_now(), e);
        }
    };

    let buy_vol_key = format!("{}:BUY:VOL", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(buy_vol_key, ts, bid_vol, buy_options.clone().label("GROUP", "VOL"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding buy vol to redis: {}", print_now(), e);
        }
    };


    let sell_options = options.clone().label("SIDE", "SELL").label("SUB", "QUOTE");
    let sell_price_key = format!("{}:SELL:PRICE", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(sell_price_key, ts, ask_price, sell_options.clone().label("GROUP", "PRICE"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding sell price to redis: {}", print_now(), e);
        }
    };

    let sell_vol_key = format!("{}:SELL:VOL", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(sell_vol_key, ts, ask_vol, sell_options.clone().label("GROUP", "VOL"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding sell vol to redis: {}", print_now(), e);
        }
    };
}

fn print_now() -> String {
    let current_datetime: DateTime<Local> = Local::now();
    current_datetime.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}

fn get_current_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis() as u64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = TsOptions::default()
        .duplicate_policy(TsDuplicatePolicy::Last)
        .retention_time(RETENTION_TIME)
        .label("EXCHANGE", "KRAKEN");

    let redis_password = env::var("REDIS_PASSWORD").expect("$REDIS_PASSWORD is not set");
    let redis_host = env::var("REDIS_HOST").unwrap_or("cache".to_string());
    let connection_string = format!("redis://default:{}@{}:6379", redis_password, redis_host);
    let client = redis::Client::open(connection_string)?;
    let mut con = client.get_connection()?;

    let expiration_duration = Duration::from_secs(30);
    let mut start_time = Instant::now();

    let (mut socket, _) = connect(Url::parse(KRAKEN_WS_API)?)?;
    println!("{}: Connected to Kraken", print_now());

    let subscription = KrakenSubscriptionMessage {
        method: "subscribe".to_string(),
        params: KrakenSubscriptionParams {
            channel: "ticker".to_string(),
            symbol: vec!["BTC/USD".to_string()],
            event_trigger: "bbo".to_string(),
            snapshot: true,
        },
    };

    let subscription_message = serde_json::to_string(&subscription)?;
    println!("{}: Sending subscription: {}", print_now(), subscription_message);
    socket.write_message(Message::Text(subscription_message))?;

    loop {
        let msg = socket.read_message();
        let message_string = match msg {
            Ok(json_str) => match json_str {
                Message::Text(s) => s,
                Message::Ping(_) => {
                    println!("{}: Received Ping", print_now());
                    socket.write_message(Message::Pong("pong".as_bytes().to_vec()))?;
                    println!("{}: Sent Pong", print_now());
                    continue;
                }
                Message::Pong(_) => {
                    println!("{}: Received Pong", print_now());
                    continue;
                }
                _ => {
                    println!("{}: Bad message: {:?}", print_now(), json_str);
                    continue;
                }
            },
            Err(error) => {
                match error {
                    tungstenite::Error::Protocol(msg) => {
                        println!("{}: Received Error::Protocol, reconnecting: {}", print_now(), msg);
                        let mut retry_count = 0;
                        let max_retries = 5;

                        while retry_count < max_retries {
                            println!("{}: Reconnection attempt {}/{}", print_now(), retry_count + 1, max_retries);

                            let delay_secs = 2_u64.pow(retry_count);
                            std::thread::sleep(Duration::from_secs(delay_secs));

                            match connect(Url::parse(KRAKEN_WS_API)?) {
                                Ok((new_socket, _)) => {
                                    socket = new_socket;
                                    println!("{}: Reconnected successfully", print_now());

                                    let subscription = KrakenSubscriptionMessage {
                                        method: "subscribe".to_string(),
                                        params: KrakenSubscriptionParams {
                                            channel: "ticker".to_string(),
                                            symbol: vec!["BTC/USD".to_string()],
                                            event_trigger: "bbo".to_string(),
                                            snapshot: true,
                                        },
                                    };
                                    let subscription_message = serde_json::to_string(&subscription)?;
                                    socket.write_message(Message::Text(subscription_message))?;
                                    break;
                                },
                                Err(e) => {
                                    println!("{}: Reconnection failed: {:?}", print_now(), e);
                                    retry_count += 1;
                                    if retry_count >= max_retries {
                                        println!("{}: Max reconnection attempts reached, exiting", print_now());
                                        return Ok(());
                                    }
                                }
                            }
                        }
                        continue;
                    },
                    _ => {
                        println!("{}: Other error: {:?}", print_now(), error);
                        continue;
                    }
                }
            }
        };


        if message_string.contains("\"method\":\"subscribe\"") {
            println!("{}: Received subscription acknowledgement", print_now());
            continue;
        }


        if message_string.contains("\"channel\":\"heartbeat\"") {
            println!("{}: Received heartbeat", print_now());
            continue;
        }

        match serde_json::from_str::<KrakenTickerMessage>(&message_string) {
            Ok(data) => {
                if data.channel == "ticker" && !data.data.is_empty() {
                    let current_timestamp = get_current_timestamp();
                    add_current_data(&mut con, current_timestamp, &data.data[0], &options);
                    start_time = Instant::now();
                }
            }
            Err(e) => {
                eprintln!("{}: Parsing Failed: {:?}", print_now(), e);
                eprintln!("{}: Message content: {}", print_now(), message_string);
            }
        }

        if start_time.elapsed() >= expiration_duration {
            println!("{}: Sending Ping", print_now());
            socket.write_message(Message::Ping("ping".as_bytes().to_vec()))?;
            start_time = Instant::now();
        }
    }
}
