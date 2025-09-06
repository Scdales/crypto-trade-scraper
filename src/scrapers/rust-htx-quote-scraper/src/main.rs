use chrono::{DateTime, Local};
use flate2::read::GzDecoder;
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json;
use std::io::Read;
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
struct HtxTickerData {
    amount: f64,
    ask: f64,
    bid: f64,
    close: f64,
    count: u64,
    high: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    low: f64,
    open: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ts: Option<u64>,
    vol: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct HtxTickerMessage {
    ch: String,
    ts: u64,
    tick: HtxTickerData,
}

#[derive(Serialize, Deserialize, Debug)]
struct HtxSubscriptionMessage {
    sub: String,
    id: String,
}

const KEY_PREFIX: &str = "HTX:XBTUSD:QUOTE";
const HTX_WS_API: &str = "wss://api.huobi.pro/ws";
const RETENTION_TIME: u64 = 3600000;

fn get_current_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis() as u64
}

fn decompress_gzip(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;
    Ok(decompressed)
}

fn add_current_data(con: &mut Connection, ts: u64, ticker: &HtxTickerData, options: &TsOptions) {
    let options_clone = options.clone().label("SIDE", "BUY").label("SUB", "QUOTE");
    let price_key = format!("{}:BUY:PRICE", KEY_PREFIX);
    let bid = ticker.bid;
    let ask = ticker.ask;
    let volume = ticker.vol;
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
    let options = TsOptions::default().duplicate_policy(TsDuplicatePolicy::Last).retention_time(RETENTION_TIME).label("EXCHANGE", "HTX");
    let redis_password = env::var("REDIS_PASSWORD").expect("$REDIS_PASSWORD is not set");
    let redis_host = env::var("REDIS_HOST").unwrap_or("cache".to_string());
    let connection_string = format!("redis://default:{}@{}:6379", redis_password, redis_host);
    let client = redis::Client::open(connection_string)?;
    let mut con = client.get_connection()?;

    let expiration_duration = Duration::from_secs(5);
    let mut start_time = Instant::now();

    let (mut socket, _) = connect(Url::parse(HTX_WS_API).unwrap()).expect("Can't connect.");

    let subscription = HtxSubscriptionMessage {
        sub: "market.btcusdt.ticker".to_string(),
        id: "id1".to_string(),
    };
    let subscription_message = serde_json::to_string(&subscription).unwrap();
    socket.write_message(Message::Text(subscription_message)).unwrap();

    loop {
        let msg: Result<Message, tungstenite::Error> = socket.read_message();
        let message_string = match msg {
            Ok(json_str) => {
                match json_str {
                    tungstenite::Message::Text(s) => s,
                    tungstenite::Message::Binary(data) => {

                        match decompress_gzip(&data) {
                            Ok(decompressed) => decompressed,
                            Err(e) => {
                                println!("{}: Failed to decompress data: {:?}", print_now(), e);
                                continue;
                            }
                        }
                    }
                    tungstenite::Message::Ping(data) => {
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
                            .write_message(Message::Pong(data))
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

                            match connect(Url::parse(HTX_WS_API).unwrap()) {
                                Ok((new_socket, _)) => {
                                    socket = new_socket;
                                    println!("{}: Reconnected successfully", print_now());


                                    let subscription = HtxSubscriptionMessage {
                                        sub: "market.btcusdt.ticker".to_string(),
                                        id: "id1".to_string(),
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


        if message_string.contains("\"ping\"") {
            if let Ok(ping_data) = serde_json::from_str::<serde_json::Value>(&message_string) {
                if let Some(ping_ts) = ping_data.get("ping") {
                    let pong_message = format!("{{\"pong\":{}}}", ping_ts);
                    socket.write_message(Message::Text(pong_message)).unwrap();
                    println!("{}: Sent pong response", print_now());
                    continue;
                }
            }
        }

        let result: Result<HtxTickerMessage, serde_json::Error> = serde_json::from_str(&message_string);
        let current_timestamp = get_current_timestamp();

        match result {
            Ok(data) => {
                add_current_data(&mut con, current_timestamp, &data.tick, &options);
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