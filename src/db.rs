use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use tokio::{
    sync::Notify,
    time::{self, Duration, Instant},
};

pub struct Db {
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<State>,
    background_task: Notify,
}

struct State {
    entries: HashMap<String, Bytes>,
    expirations: BTreeMap<Instant, String>,
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                entries: HashMap::new(),
                expirations: BTreeMap::new(),
            }),
            background_task: Notify::new(),
        });

        tokio::spawn(purge_expired_tasks(shared.clone()));

        Db { shared }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.state.lock().unwrap();
        state.entries.get(key).cloned()
    }

    pub fn set(&self, key: String, value: Bytes, expiration: Option<Duration>) {
        let mut state = self.shared.state.lock().unwrap();

        if let Some(expiration) = expiration {
            let when = Instant::now() + expiration;
            state.expirations.insert(when, key.clone());
        }

        state.entries.insert(key, value);

        self.shared.background_task.notify_one();
    }
}

impl Shared {
    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut state = self.state.lock().unwrap();

        let state = &mut *state;

        let now = Instant::now();

        while let Some((&when, key)) = state.expirations.iter().next() {
            if when > now {
                return Some(when);
            }

            state.entries.remove(key);
            state.expirations.remove(&when);
        }

        state.expirations.keys().next().cloned()
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Db {
    fn clone(&self) -> Db {
        Db {
            shared: self.shared.clone(),
        }
    }
}

async fn purge_expired_tasks(shared: Arc<Shared>) {
    loop {
        if let Some(when) = shared.purge_expired_keys() {
            tokio::select! {
                _ = time::sleep_until(when) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            shared.background_task.notified().await;
        }
    }
}
