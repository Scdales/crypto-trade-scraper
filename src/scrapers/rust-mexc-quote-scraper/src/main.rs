use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tungstenite::{connect, Message};
use url::Url;
use redis::{Connection, RedisError};
use redis_ts::{TsCommands, TsOptions, TsDuplicatePolicy};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct MexcBookTickerData {
    #[serde(rename = "bidprice")]
    bid_price: String,
    #[serde(rename = "bidquantity")]
    bid_quantity: String,
    #[serde(rename = "askprice")]
    ask_price: String,
    #[serde(rename = "askquantity")]
    ask_quantity: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MexcBookTickerMessage {
    channel: String,
    #[serde(rename = "publicbookticker")]
    public_book_ticker: MexcBookTickerData,
    symbol: String,
    #[serde(rename = "sendtime")]
    send_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct MexcSubscriptionMessage {
    method: String,
    params: Vec<String>,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct MexcErrorResponse {
    id: u64,
    code: i32,
    msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MexcSuccessResponse {
    id: u64,
    result: Option<serde_json::Value>,
}

const KEY_PREFIX: &str = "MEXC:XBTUSD:QUOTE";
const MEXC_WS_API: &str = "wss://wbs.mexc.com/ws";
const RETENTION_TIME: u64 = 3600000;

fn add_current_data(con: &mut Connection, ts: u64, ticker: &MexcBookTickerData, options: &TsOptions) {
    let bid_price: f64 = ticker.bid_price.parse().unwrap_or(0.0);
    let bid_vol: f64 = ticker.bid_quantity.parse().unwrap_or(0.0);
    let ask_price: f64 = ticker.ask_price.parse().unwrap_or(0.0);
    let ask_vol: f64 = ticker.ask_quantity.parse().unwrap_or(0.0);


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
        .label("EXCHANGE", "MEXC");

    let redis_password = env::var("REDIS_PASSWORD").expect("$REDIS_PASSWORD is not set");
    let redis_host = env::var("REDIS_HOST").unwrap_or("cache".to_string());
    let connection_string = format!("redis://default:{}@{}:6379", redis_password, redis_host);
    let client = redis::Client::open(connection_string)?;
    let mut con = client.get_connection()?;

    let expiration_duration = Duration::from_secs(5);
    let mut start_time = Instant::now();

    let (mut socket, _) = connect(Url::parse(MEXC_WS_API)?)?;
    println!("{}: Connected to MEXC", print_now());

    let subscription = MexcSubscriptionMessage {
        method: "SUBSCRIBE".to_string(),
        params: vec!["spot@public.bookTicker.v3.api@BTCUSDT".to_string()],
        id: 1,
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
                    tungstenite::Error::Protocol(_) | tungstenite::Error::AlreadyClosed => {
                        println!("{}: Connection closed/error, reconnecting: {:?}", print_now(), error);
                        let mut retry_count = 0;
                        let max_retries = 5;

                        while retry_count < max_retries {
                            println!("{}: Reconnection attempt {}/{}", print_now(), retry_count + 1, max_retries);

                            let delay_secs = 2_u64.pow(retry_count);
                            std::thread::sleep(Duration::from_secs(delay_secs));

                            match connect(Url::parse(MEXC_WS_API)?) {
                                Ok((new_socket, _)) => {
                                    socket = new_socket;
                                    println!("{}: Reconnected successfully", print_now());

                                    let subscription = MexcSubscriptionMessage {
                                        method: "SUBSCRIBE".to_string(),
                                        params: vec!["spot@public.bookTicker.v3.api@BTCUSDT".to_string()],
                                        id: 1,
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


        if message_string.contains("\"method\":\"PING\"") {
            let pong_message = "{\"method\":\"PONG\"}";
            socket.write_message(Message::Text(pong_message.to_string()))?;
            println!("{}: Sent PONG response", print_now());
            continue;
        }

        if let Ok(data) = serde_json::from_str::<MexcBookTickerMessage>(&message_string) {
            let current_timestamp = get_current_timestamp();
            add_current_data(&mut con, current_timestamp, &data.public_book_ticker, &options);
            start_time = Instant::now();
        } else if let Ok(error_resp) = serde_json::from_str::<MexcErrorResponse>(&message_string) {
            eprintln!("{}: MEXC Error Response: {} - {}", print_now(), error_resp.code, error_resp.msg);
        } else if let Ok(success_resp) = serde_json::from_str::<MexcSuccessResponse>(&message_string) {
            println!("{}: MEXC Success Response: {:?}", print_now(), success_resp.result);
        } else {
            eprintln!("{}: Parsing Failed: unable to parse as data, error, or success response", print_now());
            eprintln!("{}: Message content: {}", print_now(), message_string);
        }

        if start_time.elapsed() >= expiration_duration {
            println!("{}: Sending Ping", print_now());
            socket.write_message(Message::Ping("ping".as_bytes().to_vec()))?;
            start_time = Instant::now();
        }
    }
}
