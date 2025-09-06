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
struct BitgetTickerData {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "lastPr")]
    last_pr: String,
    #[serde(rename = "bidPr")]
    bid_pr: String,
    #[serde(rename = "askPr")]
    ask_pr: String,
    #[serde(rename = "bidSz")]
    bid_sz: String,
    #[serde(rename = "askSz")]
    ask_sz: String,
    #[serde(rename = "ts")]
    ts: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BitgetTickerMessage {
    action: String,
    arg: BitgetChannelArg,
    data: Vec<BitgetTickerData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BitgetChannelArg {
    #[serde(rename = "instType")]
    inst_type: String,
    channel: String,
    #[serde(rename = "instId")]
    inst_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BitgetSubscriptionMessage {
    op: String,
    args: Vec<BitgetChannelArg>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BitgetSubscriptionConfirmation {
    event: String,
    arg: BitgetChannelArg,
}

const KEY_PREFIX: &str = "BITGET:XBTUSD:QUOTE";

const BITGET_WS_API: &str = "wss://ws.bitget.com/v2/ws/public";

const RETENTION_TIME: u64 = 3600000;

fn add_current_data(con: &mut Connection, ts: u64, ticker: &BitgetTickerData, options: &TsOptions) {
    let options_clone = options.clone().label("SIDE", "BUY").label("SUB", "QUOTE");
    let price_key = format!("{}:BUY:PRICE", KEY_PREFIX);
    let bid: f64 = ticker.bid_pr.parse().unwrap();
    let bid_vol: f64 = ticker.bid_sz.parse().unwrap();
    let ask: f64 = ticker.ask_pr.parse().unwrap();
    let ask_vol: f64 = ticker.ask_sz.parse().unwrap();

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
    let options = TsOptions::default().duplicate_policy(TsDuplicatePolicy::Last).retention_time(RETENTION_TIME).label("EXCHANGE", "BITGET");
    let redis_password = env::var("REDIS_PASSWORD").expect("$REDIS_PASSWORD is not set");
    let redis_host = env::var("REDIS_HOST").unwrap_or("cache".to_string());
    let connection_string = format!("redis://default:{}@{}:6379", redis_password, redis_host);
    let client = redis::Client::open(connection_string)?;
    let mut con = client.get_connection()?;

    let expiration_duration = Duration::from_secs(5);
    let mut start_time = Instant::now();

    let (mut socket, _) =
        connect(Url::parse(&BITGET_WS_API).unwrap()).expect("Can't connect.");

    let subscription = BitgetSubscriptionMessage {
        op: "subscribe".to_string(),
        args: vec![BitgetChannelArg {
            inst_type: "SPOT".to_string(),
            channel: "ticker".to_string(),
            inst_id: "BTCUSDT".to_string(),
        }],
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
                            
                            match connect(Url::parse(&BITGET_WS_API).unwrap()) {
                                Ok((new_socket, _)) => {
                                    socket = new_socket;
                                    println!("{}: Reconnected successfully", print_now());


                                    let subscription = BitgetSubscriptionMessage {
                                        op: "subscribe".to_string(),
                                        args: vec![BitgetChannelArg {
                                            inst_type: "SPOT".to_string(),
                                            channel: "ticker".to_string(),
                                            inst_id: "BTCUSDT".to_string(),
                                        }],
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

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let current_timestamp = since_the_epoch.as_millis() as u64;

        if let Ok(data) = serde_json::from_str::<BitgetTickerMessage>(&message_string) {
            if !data.data.is_empty() {
                add_current_data(&mut con, current_timestamp, &data.data[0], &options);
                start_time = Instant::now();
            }
        } else if let Ok(_confirmation) = serde_json::from_str::<BitgetSubscriptionConfirmation>(&message_string) {
            println!("{}: Subscription confirmed", print_now());
        } else {
            eprintln!("{}: Parsing Failed: unable to parse as ticker or confirmation", print_now());
            eprintln!("{}: Message content: {}", print_now(), message_string);
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
