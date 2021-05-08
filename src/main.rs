extern crate redis;
use chat_proto::chat::chat_server::{Chat, ChatServer};
use chat_proto::chat::{JoinChatRoomResponse, SendChatMessageRequest, FILE_DESCRIPTOR_SET};
use chat_proto::prost_types::{FileDescriptorProto, FileDescriptorSet, Timestamp};
use chat_proto::tonic::{transport::Server, Request, Response, Status};
use chat_proto::tonic_reflection::server::Builder;
use redis::AsyncCommands;
use std::env;
use std::time::SystemTime;

// #[derive(Debug, StructOpt)]
// pub struct ServerOptions {
//    /// The address of the server that will run commands.
//    #[structopt(long, default_value = "127.0.0.1:50051")]
//    pub server_listen_addr: String,
// }

#[derive(Default)]
pub struct MyChat {}

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

    async fn join_room(&self, _: Request<()>) -> Result<Response<JoinChatRoomResponse>, Status> {
        println!("aaaaa");
        let response = JoinChatRoomResponse {
            id: 1,
            message: "aaa".to_string(),
            name: "bbb".to_string(),
            date: Some(Timestamp::from(SystemTime::now())),
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();
    let chat = MyChat::default();

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
//
// async fn fetch_an_string(mut conn: redis::aio::Connection) -> redis::RedisResult<String> {
//     let _ = conn.set("my_key", "jjjjj").await?;
//     let result = conn.get("my_key").await;
//     // conn.get("my_key").await
//     return result
// }
//
// async fn connect() -> redis::RedisResult<redis::aio::Connection> {
//     let redis_host_name =
//         env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
//     let redis_password =
//         env::var("REDIS_PASSWORD").expect("missing environment variable REDIS_PASSWORD");
//     // redisのTLS接続はredissでOK
//     // https://github.com/lettuce-io/lettuce-core/wiki/Redis-URI-and-connection-details
//     let redis_conn_url = format!("redis://:{}@{}", redis_password, redis_host_name);
//
//     redis::Client::open(redis_conn_url)
//         .expect("invalid connection URL")
//         .get_async_connection()
//         .await
// }
