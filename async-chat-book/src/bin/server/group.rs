//! A chat group

use crate::connection::Outbound;
use async_std::task;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<Arc<String>>,
}

impl Group {
    pub fn new(name: Arc<String>) -> Self {
        let (sender, _receiver) = broadcast::channel(1000);
        Self { name, sender }
    }
}
