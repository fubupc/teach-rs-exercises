use anyhow::Result;
use chat::{serialize_message, Message};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, Stdin},
    join,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    task,
};

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdin_lines = BufReader::new(stdin).lines();

    println!("Enter your username and press <enter>");
    let mut username = stdin_lines.next_line().await?.unwrap();
    username.truncate(username.trim_end().len()); // Trim newline at the end.
    let username = Message::User(username);
    println!("Connecting to server...");
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let (tcp_read, mut tcp_write) = stream.into_split();

    tcp_write.write_all(&serialize_message(username)?).await?;
    println!("Connected! You can now enter messages!");

    let chat_input_task = task::spawn(handle_chat_input(stdin_lines, tcp_write));
    let incoming_chats_task = task::spawn(handle_incoming_chats(tcp_read));
    let _ = join!(chat_input_task, incoming_chats_task);
    Ok(())
}

async fn handle_chat_input(
    mut stdin: Lines<BufReader<Stdin>>,
    mut tcp_write: OwnedWriteHalf,
) -> Result<()> {
    while let Some(line) = stdin.next_line().await? {
        let msg = Message::ClientMessage(line);
        tcp_write.write_all(&serialize_message(msg)?).await?;
    }
    Ok(())
}

async fn handle_incoming_chats(tcp_read: OwnedReadHalf) -> Result<()> {
    let mut tcp_read = BufReader::new(tcp_read).lines();
    while let Ok(Some(message)) = tcp_read.next_line().await {
        match serde_json::from_str(&message)? {
            Message::Chat { content, user } => {
                println!("<{user}>: {content}")
            }
            Message::User(username) => {
                println!("<{username}> joined the chat")
            }
            _ => {} // Let's just ignore these
        }
    }

    Ok(())
}
