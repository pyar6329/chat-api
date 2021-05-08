extern crate redis;
use chat_proto::chat::chat_server::{Chat, ChatServer};
use chat_proto::chat::{JoinChatRoomResponse, SendChatMessageRequest, FILE_DESCRIPTOR_SET};
use chat_proto::prost_types::{FileDescriptorProto, FileDescriptorSet, Timestamp};
// use chat_proto::tonic::futures_core;
use chat_proto::tonic::{transport::Server, Code, Request, Response, Status};
use chat_proto::tonic_reflection::server::Builder;
use redis::{AsyncCommands, ControlFlow, PubSubCommands};
use std::env;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

// #[derive(Debug, StructOpt)]
// pub struct ServerOptions {
//    /// The address of the server that will run commands.
//    #[structopt(long, default_value = "127.0.0.1:50051")]
//    pub server_listen_addr: String,
// }

// #[derive(Default)]
pub struct MyChat {
    redis_conn: Arc<Mutex<redis::aio::Connection>>,
    redis_conn_pubsub: Arc<Mutex<redis::aio::PubSub>>,
    redis_conn_normal: Arc<Mutex<redis::Connection>>,
}

#[chat_proto::tonic::async_trait]
impl Chat for MyChat {
    async fn send_message(
        &self,
        request: Request<SendChatMessageRequest>,
    ) -> Result<Response<()>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let a = format!("Hello {}!", request.into_inner().message);
        println!("{}", a);
        Ok(Response::new(()))
    }

    type JoinRoomStream = ReceiverStream<Result<JoinChatRoomResponse, Status>>;

    async fn join_room(
        &self,
        requeston: Request<()>,
    ) -> Result<Response<Self::JoinRoomStream>, Status> {
        let msg = JoinChatRoomResponse {
            id: 1,
            message: "aaa".to_string(),
            name: "bbb".to_string(),
            date: Some(Timestamp::from(SystemTime::now())),
        };
        println!("aaaaa");
        let msg2 = msg.clone();
        // let (mut tx, rx) = mpsc::channel(4);

        let conn_pubsub_clone = self.redis_conn_pubsub.clone();
        let channel_name = "foo".to_string();
        let mut conn_pubsub = conn_pubsub_clone.lock().await;

        let _: redis::RedisResult<()> = conn_pubsub.subscribe(channel_name).await;
        let mut stream = conn_pubsub.on_message();
        loop {
            let (mut tx, mut rx) = mpsc::channel(4);
            // while let Some(msg) = stream.next().await {
            match stream.next().await {
                Some(msg) => {
                    // let payload: String = msg.get_payload().unwrap();
                    // println!("channel '{}': {}", msg.get_channel_name(), payload);
                    // let foo_msg = JoinChatRoomResponse {
                    //     id: 1,
                    //     message: payload,
                    //     name: msg.get_channel_name().to_string(),
                    //     date: Some(Timestamp::from(SystemTime::now())),
                    // };
                    // let foo_msg2 = JoinChatRoomResponse {
                    //     id: 1,
                    //     message: "aaaaaa".to_string(),
                    //     name: msg.get_channel_name().to_string(),
                    //     date: Some(Timestamp::from(SystemTime::now())),
                    // };
                    // TODO: sendがgRPCの方に返ってこないので調査する
                    // NOTE: もしかしたらRedisのawaitの中でtokio::spawnするのかもしれない
                    // let handle = tokio::spawn(async move {
                    //     // let aaaa = tx.clone();
                    //     tx.send(Ok(foo_msg)).await.unwrap();
                    //     // match result {
                    //     //     Ok(_) => println!("okkkk"),
                    //     //     Err(_) => println!("errrrrr"),
                    //     // }
                    //     println!("send finished");
                    // });
                    // let _ = handle.await.unwrap(); // 接続中ずっとwaitする
                    tokio::spawn(async move {
                        // let aaaa = tx.clone();
                        for _ in 0..4 {
                            let payload: String = msg.get_payload().unwrap();
                            println!("channel '{}': {}", msg.get_channel_name(), payload);
                            tx.send(Ok(JoinChatRoomResponse {
                                id: 1,
                                message: payload,
                                name: msg.get_channel_name().to_string(),
                                date: Some(Timestamp::from(SystemTime::now())),
                            }))
                            .await
                            .unwrap();
                        }
                        // match result {
                        //     Ok(_) => println!("okkkk"),
                        //     Err(_) => println!("errrrrr"),
                        // }
                        println!("send finished");
                        // rx.close();
                    })
                    .await
                    .unwrap();
                    // let result_status = Status::new(Code::Ok, "");
                    // let kkk = Ok(foo_msg2);
                    // let kkk = Code::Ok(foo_msg);
                    // let kkk = Err(result_status);
                    // let nnn: Result<JoinChatRoomResponse, Status> = kkk;
                    // let ggg = ReceiverStream::new(nnn);
                    // ReceiverStream<Result<JoinChatRoomResponse, Status>>;
                    // return Ok(Response::new(ggg));
                    // return Ok(Response::new(ggg));
                    // requeston.continue();
                    // let mut streaming = requeston.into_inner();
                    // while let Some(point) = streaming.next().await {}
                    // return Ok(Response::new(ReceiverStream::new(rx)));
                    // let _ = Response::new(ReceiverStream::new(rx));
                    let _ = Response::new(rx);
                    continue;
                }
                None => {
                    // 返す値はstreamのtransaction
                    return Ok(Response::new(ReceiverStream::new(rx)));
                    // break;
                }
            }
        }
        // 返す値はstreamのtransaction
        // Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // redisのconnection
    let mut conn = connect().await?;
    let mutex_conn = Arc::new(Mutex::new(conn));
    let mut conn_normal = connect_normal().unwrap();
    let mutex_conn_normal = Arc::new(Mutex::new(conn_normal));

    let mut conn_pubsub = connect().await?;
    let mutex_conn_pubsub = Arc::new(Mutex::new(conn_pubsub.into_pubsub()));

    // gRPCのserver
    let addr = "0.0.0.0:50051".parse().unwrap();
    // let chat = MyChat::default();
    let chat = MyChat {
        redis_conn: mutex_conn,
        redis_conn_normal: mutex_conn_normal,
        redis_conn_pubsub: mutex_conn_pubsub,
    };

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

fn connect_normal() -> redis::RedisResult<redis::Connection> {
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
}
