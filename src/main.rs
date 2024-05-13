mod common;
mod group;
mod server;
mod state;

use state::RuntimeState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_util::bytes::buf::Writer;

use common::error::*;
use futures::{SinkExt, StreamExt};
use snafu::ResultExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::{runtime, spawn};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

use crate::common::utils::random_name;

const HELP_MSG: &str = include_str!("help.txt");

fn main() -> Result<(), Error> {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .worker_threads(8)
        .build()
        .unwrap();

    rt.block_on(async {
        let state = RuntimeState::new_share();
        start_server("127.0.0.1:9999".to_string(), state).await
    })?;

    println!("Hello, world!");
    Ok(())
}

async fn start_server(addr: String, state: Arc<RuntimeState>) -> Result<(), Error> {
    let tcp_listener = TcpListener::bind(addr.clone())
        .await
        .context(NetSnafu { addr })?;

    let (tx, _) = broadcast::channel::<String>(32);
    loop {
        let (stream, addr) = tcp_listener.accept().await.unwrap();
        spawn(handle_user(addr, stream, state.clone()));
    }
}

async fn handle_user(
    addr: SocketAddr,
    mut stream: TcpStream,
    state: Arc<RuntimeState>,
) -> Result<(), Error> {
    let (reader, writer) = stream.split();
    let mut frame_reader = FramedRead::new(reader, LinesCodec::new());
    let mut frame_writer = FramedWrite::new(writer, LinesCodec::new());

    //send help msg
    let _ = frame_writer.send(HELP_MSG).await;

    let mut tx = state.join_group("public".into()).await;

    let mut rx = tx.subscribe();

    //genarate random name
    let mut username = random_name();

    state
        .add_online_users(addr.to_string(), username.clone())
        .await;

    let _ = frame_writer.send(format!("my name is {username}!")).await;
    loop {
        tokio::select! {
            user_msg = frame_reader.next() => {
                if let Some(Ok(msg)) = user_msg{
                    let is_close = handle_msg(&mut username,state.clone(),msg,&mut frame_writer,&mut tx, &mut rx).await;
                    if is_close{
                        break;
                    }
                }
            },
           other_msg = rx.recv() =>{
               if let Ok(msg) = other_msg{
                 let _ = frame_writer.send(msg).await;
               }else{
                 break;
               }
            },
        }
    }
    state.remove_online_users(addr.to_string()).await;
    Ok(())
}

async fn handle_msg(
    name: &String,
    state: Arc<RuntimeState>,
    msg: String,
    frame_writer: &mut FramedWrite<tokio::net::tcp::WriteHalf<'_>, LinesCodec>,
    tx: &mut Sender<String>,
    rx: &mut Receiver<String>,
) -> bool {
    match msg.as_str() {
        "/help" => {
            let _ = frame_writer.send(HELP_MSG).await;
        }
        "/quit" => {
            let _ = frame_writer.send("ready close connection").await;
            return true;
        }
        "/list_user" => {
            let msg = state.debug_online_usres().await;
            let _ = frame_writer.send(msg).await;
        }
        _ => {
            if msg.starts_with("/join_group") {
                if let Some(group_name) = msg.split_ascii_whitespace().nth(1) {
                    let new_sender = state.join_group(group_name.to_owned()).await;
                    let new_recevier = new_sender.subscribe();
                    *rx = new_recevier;
                    *tx = new_sender;
                } else {
                    //TODO
                }
            } else {
                tx.send(format!("{name}:{msg}")).unwrap();
            }
        }
    }
    false
}

mod test {
    use std::sync::Mutex;
    use tokio::time::sleep;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_sync_lock() {
        let mtx = Mutex::new(0);

        tokio::join!(work(&mtx), work(&mtx));

        println!("{}", *mtx.lock().unwrap())
    }

    async fn work(mtx: &Mutex<i32>) {
        println!("lock");
        {
            //线程在这里被阻塞
            let mut v = mtx.lock().unwrap();
            println!("locked");
            // slow redis network request
            sleep(Duration::from_millis(100)).await;
            *v += 1;
        }
        println!("unlock")
    }
}
