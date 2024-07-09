use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use anyhow::Result;
use dashmap::DashMap;
use futures::{stream::SplitSink, SinkExt};
use tokio::{
    net::TcpStream,
    sync::{mpsc, Mutex},
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub content: String,
}

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub id: u64,
    sender: SplitSink<Framed<TcpStream, LinesCodec>, String>,
}

#[derive(Debug)]
pub struct ChatRoom {
    members: DashMap<u64, User>,
    uid: AtomicU64,
    msg_channel_sender: Arc<Mutex<mpsc::Sender<Message>>>,
    msg_channel_receiver: mpsc::Receiver<Message>,
}

impl ChatRoom {
    pub fn new() -> Self {
        let (msg_sender, msg_receiver) = mpsc::channel(128);
        Self {
            members: DashMap::new(),
            uid: AtomicU64::new(0),
            msg_channel_sender: Arc::new(Mutex::new(msg_sender)),
            msg_channel_receiver: msg_receiver,
        }
    }

    pub fn add_user(
        &self,
        name: String,
        sender: SplitSink<Framed<TcpStream, LinesCodec>, String>,
    ) -> u64 {
        let id = self.uid.fetch_add(1, Ordering::Relaxed);
        self.members.insert(id, User { name, id, sender });
        id
    }

    pub fn remove_user(&self, id: u64) {
        self.members.remove(&id);
    }

    pub async fn send_msg(&self, msg: Message) -> Result<()> {
        Ok(self.msg_channel_sender.lock().await.send(msg).await?)
    }

    pub async fn sync_msgs(&mut self) -> Result<()> {
        while let Ok(msg) = self.msg_channel_receiver.try_recv() {
            for mut user in self.members.iter_mut() {
                debug!("{}: {}", msg.sender, msg.content);
                user.value_mut()
                    .sender
                    .send(format!("{}: {}", msg.sender, msg.content))
                    .await?;
            }
        }
        Ok(())
    }
}

impl Default for ChatRoom {
    fn default() -> Self {
        Self::new()
    }
}
