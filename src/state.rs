use std::{collections::HashMap, sync::Arc};

use tokio::sync::{broadcast::Sender, RwLock};

use crate::group::Group;

pub struct RuntimeState {
    online_users: RwLock<HashMap<String, String>>,
    groups: RwLock<HashMap<String, Group>>,
}
impl RuntimeState {
    pub fn new_share() -> Arc<RuntimeState> {
        Arc::new(RuntimeState {
            online_users: RwLock::new(HashMap::new()),
            groups: RwLock::new(HashMap::new()),
        })
    }

    pub async fn add_online_users(&self, addr: String, username: String) {
        let mut name_list = self.online_users.write().await;
        name_list.insert(addr, username.clone());
    }

    pub async fn remove_online_users(&self, addr: String) {
        let mut name_list = self.online_users.write().await;
        name_list.remove(&addr);
    }

    pub async fn join_group(&self, group_name: String) -> Sender<String> {
        let mut groups = self.groups.write().await;
        let group = groups
            .entry(group_name.clone())
            .or_insert(Group::new(group_name.clone()));
        group.writer()
    }

    pub async fn debug_online_usres(&self) -> String {
        let mut msg = String::new();
        for (addr, name) in self.online_users.read().await.iter() {
            msg.push_str(&format!("addr:{},name:{}\n", addr, name));
        }
        msg
    }
}
