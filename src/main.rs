use futures::{SinkExt, StreamExt};
use snafu::prelude::Snafu;
use snafu::ResultExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::{runtime, spawn};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

const HELP_MSG: &str = include_str!("help.txt");

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("listen {addr} fail!"))]
    Net {
        source: std::io::Error,
        addr: String,
    },

    #[snafu(display("read fail"))]
    Read { source: std::io::Error },

    #[snafu(display("write fail"))]
    Write { source: std::io::Error },

    #[snafu(display("frame_writer send fail : {msg}"))]
    Frame { msg: String },
}

fn main() -> Result<(), Error> {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .worker_threads(8)
        .build()
        .unwrap();

    rt.block_on(async { start_server("127.0.0.1:9999".to_string()).await })?;

    println!("Hello, world!");
    Ok(())
}

async fn start_server(addr: String) -> Result<(), Error> {
    let tcp_listener = TcpListener::bind(addr.clone())
        .await
        .context(NetSnafu { addr })?;

    let (tx, _) = broadcast::channel::<String>(32);
    loop {
        let (stream, _) = tcp_listener.accept().await.unwrap();
        spawn(handle_user(stream, tx.clone()));
    }
}

async fn handle_user(mut stream: TcpStream, tx: Sender<String>) -> Result<(), Error> {
    let (reader, writer) = stream.split();
    let mut frame_reader = FramedRead::new(reader, LinesCodec::new());
    let mut frame_writer = FramedWrite::new(writer, LinesCodec::new());

    //send help msg
    let _ = frame_writer.send(HELP_MSG).await;
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            user_msg = frame_reader.next() => {
                if let Some(Ok(mut msg)) = user_msg{
                    if msg.starts_with("/help") {
                    let _ = frame_writer.send(HELP_MSG).await;
                    continue;
                } else if msg.starts_with("/quit") {
                    let _ = frame_writer.send("ready close connection").await;
                    break;
                }
                    //send others
                    tx.send(msg.clone()).unwrap();

                    //receive msg
                    msg.push_str("❤️");
                    frame_writer.send(msg).await.map_err(|err| Error::Frame {
                        msg: err.to_string(),
                    })?;
                }
            },
           other_msg = rx.recv() =>{
               if let Ok(msg) = other_msg{
                let _ = frame_writer.send(msg);
               }else{
                println!("asdasd");
               }
            },
        }
    }

    println!("client exit room!");
    Ok(())
}
