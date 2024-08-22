use std::sync::Arc;

use anyhow::{bail, Result};
use chat::{serialize_message, Message};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    sync::broadcast,
    task,
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

#[tokio::main]
async fn main() -> Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:8000").await?;
    let (tx, _) = broadcast::channel(1024);
    let tx = Arc::new(tx);
    loop {
        let (stream, _) = tcp_listener.accept().await?;
        let (tcp_read, tcp_write) = stream.into_split();
        println!("Connection established");

        task::spawn({
            let tx = tx.clone();
            async {
                handle_incoming(tcp_read, tx).await.ok();
            }
        });

        task::spawn({
            let rx = tx.subscribe();
            async {
                handle_outgoing(tcp_write, rx).await.ok();
            }
        });
    }
}

async fn handle_incoming(
    tcp_read: OwnedReadHalf,
    tx: impl AsRef<broadcast::Sender<Message>>,
) -> Result<()> {
    let mut tcp_read = BufReader::new(tcp_read).lines();
    let Some(initial_message) = tcp_read.next_line().await? else {
        return Ok(());
    };
    // todo!(
    //     "Deserialize initial_message into a Message::User.
    //         If the initial line is not a Message::User, stop this task."
    // );
    let init_msg: Message = serde_json::from_str(&initial_message)?;
    let Message::User(user) = init_msg.clone() else {
        bail!(
            "Expected the initial message to be Message::User, but received: {:?}",
            init_msg,
        );
    };
    let tx = tx.as_ref();
    tx.send(init_msg)?;

    // todo!("For each further incoming line, deserialize the line into a Message");
    // todo!("If the message is a Message::User, broadcast the message as-is using tx");
    // todo!(
    //     "If the message is a Message::ClientMessage,
    //     convert it into a Message::Chat and broadcast it using tx"
    // );
    // todo!("If the message is a Message::Chat, ignore it");
    while let Some(line) = tcp_read.next_line().await? {
        let msg: Message = serde_json::from_str(&line)?;
        match msg {
            Message::User(_) => tx.send(msg)?,
            Message::ClientMessage(content) => tx.send(Message::Chat {
                user: user.clone(),
                content,
            })?,
            Message::Chat { .. } => continue,
        };
    }

    Ok(())
}

async fn handle_outgoing(
    mut tcp_write: OwnedWriteHalf,
    rx: broadcast::Receiver<Message>,
) -> Result<()> {
    let mut rx = BroadcastStream::from(rx);
    while let Some(Ok(msg)) = rx.next().await {
        // todo!(
        //     "Serialize message as JSON and send it to the client,
        //     along with a newline"
        // );
        tcp_write.write_all(&serialize_message(msg)?).await?;
    }
    Ok(())
}
