use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    broadcast::{self, Sender},
    RwLock,
};

pub struct Group {
    name: String,
    users: Arc<RwLock<HashMap<String, String>>>,
    msg_writer: Sender<String>,
}

impl Group {
    pub fn new(name: String) -> Group {
        let (msg_writer, _) = broadcast::channel(32);
        let users = Arc::new(RwLock::new(HashMap::new()));
        Self {
            name,
            users,
            msg_writer,
        }
    }

    pub async fn join(&self, username: String) -> Sender<String> {
        self.msg_writer.clone()
    }

    pub fn writer(&self) -> Sender<String> {
        self.msg_writer.clone()
    }
}
