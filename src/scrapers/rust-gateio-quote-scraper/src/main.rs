use chrono::{DateTime, Local};
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json;
use tungstenite::{connect, Message};
use url::Url;
use redis::{Connection, RedisError};
use redis_ts::{TsCommands, TsOptions, TsDuplicatePolicy};
use std::env;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn de_float_from_str<'a, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'a>,
{
    let str_val = String::deserialize(deserializer)?;
    str_val.parse::<f64>().map_err(de::Error::custom)
}

#[derive(Serialize, Deserialize, Debug)]
struct GateioTickerData {
    currency_pair: String,
    last: String,
    lowest_ask: String,
    highest_bid: String,
    change_percentage: String,
    base_volume: String,
    quote_volume: String,
    high_24h: String,
    low_24h: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GateioTickerMessage {
    time: u64,
    channel: String,
    event: String,
    result: GateioTickerData,
}

#[derive(Serialize, Deserialize, Debug)]
struct GateioSubscriptionMessage {
    time: u64,
    channel: String,
    event: String,
    payload: Vec<String>,
}

const KEY_PREFIX: &str = "GATEIO:XBTUSD:QUOTE";
const GATEIO_WS_API: &str = "wss://api.gateio.ws/ws/v4/";
const RETENTION_TIME: u64 = 3600000;

fn get_current_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis() as u64
}

fn add_current_data(con: &mut Connection, ts: u64, ticker: &GateioTickerData, options: &TsOptions) {
    let options_clone = options.clone().label("SIDE", "BUY").label("SUB", "QUOTE");
    let price_key = format!("{}:BUY:PRICE", KEY_PREFIX);
    let bid: f64 = ticker.highest_bid.parse().unwrap();
    let ask: f64 = ticker.lowest_ask.parse().unwrap();

    let volume: f64 = ticker.base_volume.parse().unwrap_or(0.0);
    let bid_vol = volume / 2.0;
    let ask_vol = volume / 2.0;

    let redis_query: Result<(), RedisError> = con.ts_add_create(price_key, ts, bid, options_clone.clone().label("GROUP", "PRICE"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding buy price to redis: {}", print_now(), e);
        }
    };
    let vol_key = format!("{}:BUY:VOL", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(vol_key, ts, bid_vol, options_clone.clone().label("GROUP", "VOL"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding buy vol to redis: {}", print_now(), e);
        }
    };


    let options_clone = options.clone().label("SIDE", "SELL").label("SUB", "QUOTE");
    let price_key = format!("{}:SELL:PRICE", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(price_key, ts, ask, options_clone.clone().label("GROUP", "PRICE"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding sell price to redis: {}", print_now(), e);
        }
    };
    let vol_key = format!("{}:SELL:VOL", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(vol_key, ts, ask_vol, options_clone.clone().label("GROUP", "VOL"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding sell vol to redis: {}", print_now(), e);
        }
    };
}

fn print_now() -> String {
     let current_datetime: DateTime<Local> = Local::now();
     let formatted_datetime = current_datetime.format("%Y-%m-%d %H:%M:%S%.6f").to_string();
     formatted_datetime
}

fn main() -> redis::RedisResult<()> {
    let options = TsOptions::default().duplicate_policy(TsDuplicatePolicy::Last).retention_time(RETENTION_TIME).label("EXCHANGE", "GATEIO");
    let redis_password = env::var("REDIS_PASSWORD").expect("$REDIS_PASSWORD is not set");
    let redis_host = env::var("REDIS_HOST").unwrap_or("cache".to_string());
    let connection_string = format!("redis://default:{}@{}:6379", redis_password, redis_host);
    let client = redis::Client::open(connection_string)?;
    let mut con = client.get_connection()?;

    let expiration_duration = Duration::from_secs(5);
    let mut start_time = Instant::now();

    let (mut socket, _) = connect(Url::parse(GATEIO_WS_API).unwrap()).expect("Can't connect.");

    let subscription = GateioSubscriptionMessage {
        time: get_current_timestamp() / 1000,
        channel: "spot.tickers".to_string(),
        event: "subscribe".to_string(),
        payload: vec!["BTC_USDT".to_string()],
    };
    let subscription_message = serde_json::to_string(&subscription).unwrap();
    socket.write_message(Message::Text(subscription_message)).unwrap();

    loop {
        let msg: Result<Message, tungstenite::Error> = socket.read_message();
        let message_string = match msg {
            Ok(json_str) => {
                match json_str {
                    tungstenite::Message::Text(s) => s,
                    tungstenite::Message::Ping(_) => {
                        loop {
                            let is_pending = socket.write_pending();
                            match is_pending {
                                Ok(_) => {
                                    break;
                                }
                                Err(v) => println!("{}: Write Pending Error: {:?}", print_now(), v),
                            };
                        }
                        println!("{}: Received Ping", print_now());
                        socket
                            .write_message(Message::Pong("pong".as_bytes().to_vec()))
                            .unwrap();
                        println!("{}: Sent Pong", print_now());
                        continue;
                    },
                    tungstenite::Message::Pong(_) => {
                        println!("{}: Received Pong", print_now());
                        continue;
                    },
                    _ => {
                        println!("{}: Bad message: {:?}", print_now(), json_str.to_string());
                        continue;
                    }
                }
            }
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

                            match connect(Url::parse(GATEIO_WS_API).unwrap()) {
                                Ok((new_socket, _)) => {
                                    socket = new_socket;
                                    println!("{}: Reconnected successfully", print_now());


                                    let subscription = GateioSubscriptionMessage {
                                        time: get_current_timestamp() / 1000,
                                        channel: "spot.tickers".to_string(),
                                        event: "subscribe".to_string(),
                                        payload: vec!["BTC_USDT".to_string()],
                                    };
                                    let subscription_message = serde_json::to_string(&subscription).unwrap();
                                    socket.write_message(Message::Text(subscription_message)).unwrap();
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


        if !message_string.contains("spot.tickers") {
            continue;
        }

        let result: Result<GateioTickerMessage, serde_json::Error> = serde_json::from_str(&message_string);
        let current_timestamp = get_current_timestamp();

        match result {
            Ok(data) => {
                add_current_data(&mut con, current_timestamp, &data.result, &options);
                start_time = Instant::now();
            }
            Err(e) => {
                eprintln!("{}: Parsing Failed: {:?}", print_now(), e);
                eprintln!("{}: Message content: {}", print_now(), message_string);
            }
        }

        if start_time.elapsed() >= expiration_duration {
            println!("{}: Sending Ping", print_now());
            socket
                .write_message(Message::Ping("ping".as_bytes().to_vec()))
                .unwrap();
            start_time = Instant::now();
        }
    }
}