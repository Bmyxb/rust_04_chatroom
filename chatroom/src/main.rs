use std::{sync::Arc, thread::sleep, time::Duration};
use tokio::sync::Mutex;

use anyhow::Result;
use chatroom::{ChatRoom, Message};
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn handle_stream(stream: TcpStream, chat_room: Arc<Mutex<ChatRoom>>) -> Result<()> {
    let stream = Framed::new(stream, LinesCodec::new());
    let (mut sender, mut receiver) = stream.split::<String>();

    sender.send("Enter your username:".to_owned()).await?;
    let user_name = receiver.next().await.unwrap()?;
    let uid = chat_room.lock().await.add_user(user_name.clone(), sender);

    while let Some(Ok(line)) = receiver.next().await {
        let msg = line.trim();
        if msg.is_empty() {
            continue;
        } else if msg == "quit" {
            chat_room.lock().await.remove_user(uid);
            break;
        }

        chat_room
            .lock()
            .await
            .send_msg(Message {
                sender: user_name.clone(),
                content: msg.to_owned(),
            })
            .await?;
    }

    Ok(())
}

pub async fn broadcast_msgs(chat_room: Arc<Mutex<ChatRoom>>) -> Result<()> {
    loop {
        let mut chat_room = chat_room.lock().await;
        chat_room.sync_msgs().await?;
        sleep(Duration::from_millis(100));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let chat_room = Arc::new(Mutex::new(ChatRoom::new()));
    let addr = "0.0.0.0:8000";
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from {}", addr);
        let cloned = Arc::clone(&chat_room);
        tokio::spawn(async move {
            broadcast_msgs(cloned).await.unwrap();
        });

        let cloned = Arc::clone(&chat_room);
        tokio::spawn(async move {
            let _ = handle_stream(stream, cloned).await;
        });
    }
}
