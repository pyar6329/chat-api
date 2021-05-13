extern crate redis;
use chat_proto::chat::chat_server::{Chat, ChatServer};
use chat_proto::chat::{JoinChatRoomResponse, SendChatMessageRequest, FILE_DESCRIPTOR_SET};
use chat_proto::prost_types::{FileDescriptorProto, FileDescriptorSet, Timestamp};
// use chat_proto::tonic::futures_core;
use chat_proto::tonic::{transport::Server, Code, Request, Response, Status};
use chat_proto::tonic_reflection::server::Builder;
use redis::aio::Connection as RedisConnection;
use redis::aio::ConnectionManager as RedisConnectionManager;
use redis::aio::MultiplexedConnection as RedisMultiplexedConnection;
use redis::aio::PubSub as RedisPubSub;
// use redis::AsyncCommands;
// use redis::Commands;
use redis::IntoConnectionInfo as RedisConnectionInfo;
use redis::RedisError;
use redis::{AsyncCommands, ControlFlow, PubSubCommands};
use std::env;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
// use tokio_stream::wrappers::ReceiverStream;
// use tokio_stream::{self, StreamExt};

// #[derive(Debug, StructOpt)]
// pub struct ServerOptions {
//    /// The address of the server that will run commands.
//    #[structopt(long, default_value = "127.0.0.1:50051")]
//    pub server_listen_addr: String,
// }

// #[derive(Default)]
pub struct MyChat {}

#[chat_proto::tonic::async_trait]
impl Chat for MyChat {
    async fn send_message(
        &self,
        request: Request<SendChatMessageRequest>,
    ) -> Result<Response<()>, Status> {
        // println!("Got a request from {:?}", request.remote_addr());
        // let a = format!("Hello {}!", request.into_inner().message);
        // println!("{}", a);
        let request_inner = request.into_inner();

        let (tx, mut rx) = mpsc::channel(4);
        let channel_name = "foo".to_string();

        tokio::spawn(async move {
            // 1 request毎にredis pub/subのconnectionを貼る
            let conn = &mut connect().await.unwrap();

            // let mut bbb = conn.clone();
            let result: Result<(), RedisError> =
                conn.publish(channel_name, request_inner.message).await;
            let response_result = result.map_or_else(
                |fail| Status::aborted(fail.to_string()),
                |_| Status::ok("succeed"),
            );
            if let Err(_) = tx.send(response_result).await {
                println!("send was failed.");
            }
            println!("send finished");
        });
        // 返す値はstreamのtransaction
        // Ok(Response::new(ReceiverStream::new(rx)))
        // Ok(Response::new(()))
        match rx.recv().await {
            Some(status) if status.code() == Code::Ok => Ok(Response::new(())),
            Some(_) => {
                println!("redis publish failed");
                Err(Status::aborted("something worng."))
            }
            None => {
                println!("tx send failed");
                Err(Status::aborted("something worng."))
            }
        }
    }

    type JoinRoomStream = ReceiverStream<Result<JoinChatRoomResponse, Status>>;

    async fn join_room(&self, _: Request<()>) -> Result<Response<Self::JoinRoomStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let channel_name = "foo".to_string();
        // spawnは非同期にしないとOkで値を返せないので、awaitしないこと
        tokio::spawn(async move {
            // 1 request毎にredis pub/subのconnectionを貼る
            let mut conn_pubsub = connect_pubsub().await.unwrap();
            let _ = conn_pubsub.subscribe(channel_name).await;
            let mut stream = conn_pubsub.on_message();
            // 非同期でloopしないと、returnで値を返せない
            while let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload().unwrap();
                println!("channel '{}': {}", msg.get_channel_name(), payload);
                let send_msg = Ok(JoinChatRoomResponse {
                    id: 1,
                    message: payload, // TODO: ここのpayloadをjsonで管理する
                    name: msg.get_channel_name().to_string(),
                    date: Some(Timestamp::from(SystemTime::now())),
                });
                // 送信失敗したら非同期処理を終わる
                if let Err(_) = tx.send(send_msg).await {
                    println!("send was failed");
                    break;
                }
                println!("send finished");
            }
        });
        // 返す値はstreamのtransaction
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // redisのconnection
    // let mut conn = connect().await?;
    // let mutex_conn = Arc::new(Mutex::new(conn));
    // let mut conn_normal = connect_normal().unwrap();
    // let mutex_conn_normal = Arc::new(Mutex::new(conn_normal));
    //
    // let mut conn_pubsub = connect().await?;
    // let mutex_conn_pubsub = Arc::new(Mutex::new(conn_pubsub.into_pubsub()));

    // gRPCのserver
    let addr = "0.0.0.0:50051".parse().unwrap();
    // let chat = MyChat::default();
    let chat = MyChat {};

    println!("ChatServer listening on {}", addr);

    // gRPC reflection server creating
    let reflection = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .add_service(reflection)
        .add_service(ChatServer::new(chat))
        .serve(addr)
        .await?;
    Ok(())
}

// ----- redis -----

// #[tokio::main]
// async fn main() -> redis::RedisResult<()> {
//     let mut conn = connect().await?;
//     // conn.set("my_key", "bbbbb").await?;
//     let result = fetch_an_string(conn).await?;
//     println!("{}", result);
//     println!("Hello, world!");
//     Ok(())
// }

// async fn fetch_an_string(mut conn: redis::aio::Connection) -> redis::RedisResult<String> {
//     let _ = conn.set("my_key", "jjjjj").await?;
//     let result = conn.get("my_key").await;
//     // conn.get("my_key").await
//     return result;
// }

async fn fetch_an_string(
    mut conn: tokio::sync::MutexGuard<'_, redis::aio::Connection>,
) -> redis::RedisResult<String> {
    let _ = conn.set("my_key", "jjjjj").await?;
    let result = conn.get("my_key").await;
    // conn.get("my_key").await
    return result;
}

// connection using ConnectionManager
async fn connect() -> Result<RedisConnectionManager, RedisError> {
    // async fn connect() -> Result<RedisMultiplexedConnection, RedisError> {
    // async fn connect() -> Result<RedisConnection, RedisError> {
    let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
    let redis_password =
        env::var("REDIS_PASSWORD").expect("missing environment variable REDIS_PASSWORD");
    // redisのTLS接続はredissでOK
    // https://github.com/lettuce-io/lettuce-core/wiki/Redis-URI-and-connection-details
    let redis_conn_url = format!("redis://:{}@{}", redis_password, redis_host_name);

    let client = redis::Client::open(redis_conn_url)?;
    // let conn = client.get_async_connection().await;
    // let conn = client.get_multiplexed_async_connection().await;
    // let connection_manager = client.get_tokio_connection_manager().await.unwrap();
    let conn = client.get_tokio_connection_manager().await;
    // let conn = RedisConnectionManager::new(client).await;
    conn
}

async fn connect_pubsub() -> Result<RedisPubSub, RedisError> {
    let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
    let redis_password =
        env::var("REDIS_PASSWORD").expect("missing environment variable REDIS_PASSWORD");
    // redisのTLS接続はredissでOK
    // https://github.com/lettuce-io/lettuce-core/wiki/Redis-URI-and-connection-details
    let redis_conn_url = format!("redis://:{}@{}", redis_password, redis_host_name);

    let client = redis::Client::open(redis_conn_url)?;
    let conn = client.get_async_connection().await?;
    Ok(conn.into_pubsub())
}

// #[derive(Clone)]
// pub struct PubSubConnectionPool {
//     conn: RedisPubSub,
// }
//
// impl PubSubConnectionPool {
//     // async fn new<T: RedisConnectionInfo>(params: T) -> Result<RedisPubSub, RedisError> {
//     async fn new() -> Result<PubSubConnectionPool, RedisError> {
//         let redis_host_name =
//             env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
//         let redis_password =
//             env::var("REDIS_PASSWORD").expect("missing environment variable REDIS_PASSWORD");
//         // redisのTLS接続はredissでOK
//         // https://github.com/lettuce-io/lettuce-core/wiki/Redis-URI-and-connection-details
//         let redis_conn_url = format!("redis://:{}@{}", redis_password, redis_host_name);
//
//         let client = redis::Client::open(redis_conn_url)?;
//         let conn = client.get_async_connection().await?;
//         Ok(PubSubConnectionPool {
//             conn: conn.into_pubsub(),
//         })
//     }
// }

// #[derive(Clone)]
// pub struct ConnectionPool {
//     conn: &mut &'static RedisMultiplexedConnection,
// }
//
// impl ConnectionPool {
//     fn new(conn: &mut &'static RedisMultiplexedConnection) -> ConnectionPool {
//         ConnectionPool { conn }
//     }
//
//     pub async fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, RedisError> {
//         self.conn.get(key).await
//     }
//
//     pub async fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), RedisError> {
//         self.conn.set(key, value).await
//     }
//
//     pub async fn publish(&mut self, channel: &[u8], message: &[u8]) -> Result<(), RedisError> {
//         self.conn.publish(channel, message).await
//     }
// }
