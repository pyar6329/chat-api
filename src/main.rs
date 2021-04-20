extern crate redis;
use std::env;
use redis::AsyncCommands;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let mut conn = connect().await?;
    // conn.set("my_key", "bbbbb").await?;
    let result = fetch_an_string(conn).await?;
    println!("{}", result);
    println!("Hello, world!");
    Ok(())
}

async fn fetch_an_string(mut conn: redis::aio::Connection) -> redis::RedisResult<String> {
    let _ = conn.set("my_key", "jjjjj").await?;
    let result = conn.get("my_key").await;
    // conn.get("my_key").await
    return result
}

async fn connect() -> redis::RedisResult<redis::aio::Connection> {
    let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
    let redis_password =
        env::var("REDIS_PASSWORD").expect("missing environment variable REDIS_PASSWORD");
    // redisのTLS接続はredissでOK
    // https://github.com/lettuce-io/lettuce-core/wiki/Redis-URI-and-connection-details
    let redis_conn_url = format!("redis://:{}@{}", redis_password, redis_host_name);

    redis::Client::open(redis_conn_url)
        .expect("invalid connection URL")
        .get_async_connection()
        .await
}
