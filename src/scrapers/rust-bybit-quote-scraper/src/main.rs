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
    if str_val.is_empty() {
        return Err(de::Error::custom("cannot parse float from empty string"));
    }
    str_val.parse::<f64>().map_err(de::Error::custom)
}

#[derive(Serialize, Deserialize, Debug)]
struct BybitMessageQuoteData {
    symbol: String,
    #[serde(rename = "lastPrice", deserialize_with = "de_float_from_str")]
    last_price: f64,
    #[serde(rename = "highPrice24h", deserialize_with = "de_float_from_str")]
    high_price24h: f64,
    #[serde(rename = "lowPrice24h", deserialize_with = "de_float_from_str")]
    low_price24h: f64,
    #[serde(rename = "prevPrice24h", deserialize_with = "de_float_from_str")]
    prev_price24h: f64,
    #[serde(rename = "volume24h", deserialize_with = "de_float_from_str")]
    volume24h: f64,
    #[serde(rename = "turnover24h", deserialize_with = "de_float_from_str")]
    turnover24h: f64,
    #[serde(rename = "price24hPcnt", deserialize_with = "de_float_from_str")]
    price24h_pcnt: f64,
    #[serde(rename = "usdIndexPrice", deserialize_with = "de_float_from_str")]
    usd_index_price: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct BybitMessageQuote {
    topic: String, // tickers.BTCUSDT,
    ts: u64, // 1708270510698,
    r#type: String, // snapshot,
    cs: u64, // 23880169860,
    data: BybitMessageQuoteData
}


#[derive(Serialize, Deserialize, Debug)]
struct BybitSubscriptionMessage {
    op: String,
    args: Vec<String>
}

const KEY_PREFIX: &str = "BYBIT:XBTUSD:QUOTE";

const BYBIT_WS_API: &str = "wss://stream.bybit.com/v5/public/spot";

const RETENTION_TIME: u64 = 3600000;

fn add_current_data(con: &mut Connection, ts: u64, quote: &BybitMessageQuote, options: &TsOptions) {

    let options_clone = options.clone().label("SIDE", "BUY").label("SUB", "QUOTE");
    let price_key = format!("{}:BUY:PRICE", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(price_key, ts, quote.data.last_price, options_clone.clone().label("GROUP", "PRICE"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding buy price to redis: {}", print_now(), e);
        }
    };
    let vol_key = format!("{}:BUY:VOL", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(vol_key, ts, quote.data.volume24h, options_clone.clone().label("GROUP", "VOL"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding buy vol to redis: {}", print_now(), e);
        }
    };

    let options_clone = options.clone().label("SIDE", "SELL").label("SUB", "QUOTE");
    let price_key = format!("{}:SELL:PRICE", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(price_key, ts, quote.data.last_price, options_clone.clone().label("GROUP", "PRICE"));
    match redis_query {
        Ok(_) => {},
        Err(e) => {
            println!("{}: Error adding sell price to redis: {}", print_now(), e);
        }
    };
    let vol_key = format!("{}:SELL:VOL", KEY_PREFIX);
    let redis_query: Result<(), RedisError> = con.ts_add_create(vol_key, ts, quote.data.volume24h, options_clone.clone().label("GROUP", "VOL"));
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
    let options = TsOptions::default().duplicate_policy(TsDuplicatePolicy::Last).retention_time(RETENTION_TIME).label("EXCHANGE", "BYBIT");
    let redis_password = env::var("REDIS_PASSWORD").expect("$REDIS_PASSWORD is not set");
    let redis_host = env::var("REDIS_HOST").unwrap_or("cache".to_string());
    let connection_string = format!("redis://default:{}@{}:6379", redis_password, redis_host);
    let client = redis::Client::open(connection_string)?;
    let mut con = client.get_connection()?;

    let expiration_duration = Duration::from_secs(5);
    let mut start_time = Instant::now();

    let (mut socket, _) =
        connect(Url::parse(&BYBIT_WS_API).unwrap()).expect("Can't connect.");
    println!("Connected");
    let subscription = BybitSubscriptionMessage {
        op: String::from("subscribe"),
        args: vec![String::from("tickers.BTCUSDT")]
    };
    let subscription_message = serde_json::to_string::<BybitSubscriptionMessage>(&subscription).unwrap();
    println!("Sending: {:?}", subscription_message);
    socket
        .write_message(Message::from(subscription_message))
        .unwrap();
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
                            
                            match connect(Url::parse(&BYBIT_WS_API).unwrap()) {
                                Ok((new_socket, _)) => {
                                    socket = new_socket;
                                    println!("{}: Reconnected successfully", print_now());
                                    

                                    let subscription = BybitSubscriptionMessage {
                                        op: String::from("subscribe"),
                                        args: vec![String::from("tickers.BTCUSDT")]
                                    };
                                    let subscription_message = serde_json::to_string::<BybitSubscriptionMessage>(&subscription).unwrap();
                                    println!("{}: Re-subscribing: {:?}", print_now(), subscription_message);
                                    
                                    match socket.write_message(Message::from(subscription_message)) {
                                        Ok(_) => {
                                            println!("{}: Re-subscribed successfully", print_now());
                                            break;
                                        },
                                        Err(e) => {
                                            println!("{}: Failed to re-subscribe: {:?}", print_now(), e);
                                            retry_count += 1;
                                            continue;
                                        }
                                    }
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
        let result: Result<BybitMessageQuote, serde_json::Error> = serde_json::from_str(&message_string);
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let current_timestamp = since_the_epoch.as_millis() as u64;

        match result {
            Ok(data) => {
                add_current_data(&mut con, current_timestamp, &data, &options);
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
