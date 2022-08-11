use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8082").await.unwrap();

    let (sender, rec) = broadcast::channel::<(String, SocketAddr)>(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        let sender = sender.clone();
        let mut rec = rec.resubscribe();


        tokio::spawn(async move {
            let (read_half, mut write_half) = socket.split();

            let mut buf_reader = BufReader::new(read_half);
            let mut line = String::new();
            loop {
                tokio::select! {
                    result = buf_reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        sender.send((line.clone(), addr)).unwrap();
                        line.clear()
                    }
                    result = rec.recv() => {
                        let (msg, other_addr) = result.unwrap();

                        if addr != other_addr {
                            write_half.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
