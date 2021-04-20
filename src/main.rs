extern crate redis;
use std::env;
use redis::{Commands, Connection};

fn main() {
    println!("{}", fetch_an_string());
    println!("Hello, world!");
}

fn fetch_an_string() -> String {

    let mut conn = connect();
    // throw away the result, just make sure it does not fail
    let _ : () = conn.set("my_key", "bbbbb").expect("failed set my_key");
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    let result : String = conn.get("my_key").unwrap();
    return result;
}

fn connect() -> redis::Connection {
    let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
    let redis_password =
        env::var("REDIS_PASSWORD").expect("missing environment variable REDIS_PASSWORD");
    // redisのTLS接続はredissでOK
    // https://github.com/lettuce-io/lettuce-core/wiki/Redis-URI-and-connection-details
    let redis_conn_url = format!("redis://:{}@{}", redis_password, redis_host_name);

    redis::Client::open(redis_conn_url)
        .expect("invalid connection URL")
        .get_connection()
        .expect("failed to connect to redis")
}
