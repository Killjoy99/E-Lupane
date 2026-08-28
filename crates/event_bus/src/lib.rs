use tokio::sync::broadcast;


#[derive(Clone, Debug)]
pub enum SystemEvent {
    PluginLoaded(String),
    PluginEnabled(String),
    PluginDisabled(String),
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<String>,
}

impl EventBus {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn publish(&self, event: String) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }
}