#[forbid(unsafe_code)]
#[forbid(unused_imports)]
#[forbid(missing_docs)]
#[cfg(test)]
mod test;
#[cfg(feature = "derive")]
pub mod derive;

use crossbeam_channel::{Receiver, Sender};
use diff::Diff;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use spin_sleep::sleep;
use log::{info, debug, trace};

pub type Result<T> = std::result::Result<T, Error>;

/// Library error.
#[derive(Error, Debug)]
pub enum Error {
    /// Incorrect pattern for variables
    #[error("Invalid pattern: {pattern:?}. Error: {error:?}")]
    InvalidPattern { pattern: String, error: String },

    #[error("Re-init env watcher.")]
    DoubleInitialWatcher,

    #[error("In current watcher exists subscribers.")]
    ReinitializedWithSubscribers,
}

/// Changing the current state for a subscriber
#[derive(Debug, Clone)]
pub enum ChangeState {
    /// Add or change state
    Edit(String, String),

    /// Delete key
    Delete(String),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Subscribe {
    /// Return all env variables
    All,

    /// Subscribe by env list
    /// let v = vec!["my.project.key1", "my.project.key2", "my.project.key3"];
    /// let subscribe = Subscribe::Envs(v);
    Envs(Vec<String>),

    /// Subscribe by pattern env.
    /// Example by pattern:
    /// let v = vec!["my.project.*", "my.project2.*"];
    /// let subscribe = Subscribe::PatternEnvs(v);
    PatternEnvs(Vec<String>),
}

/// Baseline implementation for data.
/// A separate thread listens for data changes through the channel, in case of data changes, we receive an event and change the data snapshot.
pub struct EnvironmentData {
    /// Snapshot data
    data: Arc<Mutex<HashMap<String, String>>>,

    /// Channel for receiving changes for a specific key
    rx: Receiver<ChangeState>,
}

impl EnvironmentData {
    /// Getter for snapshot data
    pub fn data(&self) -> HashMap<String, String> {
        self.data.lock().unwrap().clone()
    }

    /// Reference for current snapshot
    pub fn ref_data(&self) -> Arc<Mutex<HashMap<String, String>>> {
        Arc::clone(&self.data)
    }

    /// In a separate thread, we listen to the change of variables
    pub fn receive(&self) {
        let snapshot = Arc::clone(&self.data);
        let rx = self.rx.clone();

        std::thread::spawn(move || loop {
            let data = rx.recv().unwrap();
            let mut snapshot = snapshot.lock().unwrap();
            match data {
                ChangeState::Edit(k, v) => {
                    snapshot.insert(k.clone(), v.clone()).unwrap();
                }
                ChangeState::Delete(k) => {
                    snapshot.remove(&*k).unwrap();
                }
            };
        });
    }
}

/// The current state of the environment
pub struct EnvironmentWatcher {
    /// Current env state
    state: Arc<Mutex<HashMap<String, String>>>,

    /// Sender list
    /// key - subscribe type
    /// value - sender list, for notification
    senders: Arc<Mutex<HashMap<Subscribe, Vec<Sender<ChangeState>>>>>,

    /// reading environment variables
    interval: Duration,
}

impl EnvironmentWatcher {
    /// Create a new instance to track the state
    /// Interval - how often we request data and update the state (if required)
    pub fn new(interval: Duration) -> Self {
        info!("Starting env watcher with interval {:?}", &interval);
        let env_state = Self {
            state: Arc::new(Mutex::new(Default::default())),
            senders: Arc::new(Mutex::new(Default::default())),
            interval,
        };
        env_state.preload();
        env_state.run();
        env_state
    }

    /// Preload the environment
    fn preload(&self) {
        let mut data = self.state.lock().unwrap();
        std::env::vars().for_each(|kv| {
            data.insert(kv.0, kv.1);
        });
        trace!("Preload environment map:\n{:?}", &data)
    }

    /// Subscribers size. (only `Subscribe`)
    pub fn size(&self) -> usize {
        let size = self.senders.lock().unwrap().len();
        debug!("Current subscribers size: {:?}", &size);
        size
    }

    /// Subscribe to the keys and get a snapshot of the data
    pub fn subscribe_snapshot(&self, subscribe: Subscribe) -> Result<EnvironmentData> {
        let sub = self.subscribe(subscribe)?;
        let data = EnvironmentData {
            data: Arc::new(Mutex::new(sub.0)),
            rx: sub.1,
        };
        data.receive();
        Ok(data)
    }

    /// We subscribe to the keys, if successful, we get a snapshot of the current data and a channel for updating this data
    pub fn subscribe(
        &self,
        subscribe: Subscribe,
    ) -> Result<(HashMap<String, String>, Receiver<ChangeState>)> {
        debug!("Subscribe by {:?}", &subscribe);
        let (tx, rx) = crossbeam_channel::unbounded::<ChangeState>();

        let mut data = {
            let state = self.state.lock();

            let state_guard = state.unwrap();

            state_guard.clone()
        };

        let sub = match &subscribe {
            Subscribe::All => (data, rx),

            Subscribe::Envs(envs) => {
                data.retain(|k, _| envs.contains(k));

                (data, rx)
            }

            Subscribe::PatternEnvs(envs) => {
                let envs = envs
                    .iter()
                    .map(|pattern| {
                        Regex::new(&*pattern)
                            .map_err(|e| Error::InvalidPattern {
                                pattern: pattern.clone(),
                                error: e.to_string(),
                            })
                            .unwrap()
                    })
                    .collect::<Vec<Regex>>();

                data.retain(|k, _| {
                    let mut find = false;
                    for env in envs.iter() {
                        match env.find(k) {
                            None => {}
                            Some(_) => {
                                find = true;
                            }
                        }

                        if find {
                            break;
                        }
                    }
                    find
                });

                (data, rx)
            }
        };

        self._subscribe(subscribe.clone(), tx);
        Ok(sub)
    }

    /// Adding keys to the current state.
    fn _subscribe(&self, sub: Subscribe, tx: Sender<ChangeState>) {
        let senders = self.senders.lock();
        let mut guard = senders.unwrap();
        let entry = guard.entry(sub).or_insert_with(|| vec![]);
        entry.push(tx);
    }

    /// In a separate thread, we process state changes at intervals.
    /// If the values change, we will notify the subscribers who have subscribed to these values.
    pub fn run(&self) {
        let data = Arc::clone(&self.state);
        let subs = Arc::clone(&self.senders);
        let interval = self.interval.clone();

        std::thread::spawn(move || loop {
            {
                let data = data.lock();
                let mut data_guard = data.unwrap();

                let subs = subs.lock();
                let mut subs_guard = subs.unwrap();

                let mut sys_data = HashMap::<String, String>::new();
                std::env::vars().for_each(|kv| {
                    sys_data.insert(kv.0, kv.1);
                });

                if !sys_data.eq(&data_guard) {
                    let different = data_guard.diff(&sys_data);

                    let mut changes = HashMap::<String, ChangeState>::new();

                    let remove_set = different.removed;
                    let altered = different.altered;

                    if !remove_set.is_empty() {
                        remove_set.iter().for_each(|k| {
                            let delete = ChangeState::Delete(k.clone());
                            changes.insert(k.clone(), delete);
                        });
                    }

                    if !altered.is_empty() {
                        altered.iter().for_each(|k| {
                            let alter =
                                ChangeState::Edit(k.0.clone(), k.1.clone().unwrap_or_default());
                            changes.insert(k.0.clone(), alter);
                        });
                    }

                    if !changes.is_empty() {
                        debug!("Find changes in environment.\nDiff {:?}", &changes);
                        subs_guard.iter_mut().for_each(|s| {
                            let sub = s.0;
                            let senders = s.1;

                            match sub {
                                Subscribe::All => {
                                    changes.iter().for_each(|change| {
                                        senders.iter().for_each(|sender| {
                                            sender.send(change.1.clone()).unwrap();
                                        });
                                    });
                                }

                                Subscribe::Envs(envs) => {
                                    changes.iter().for_each(|change| {
                                        if envs.contains(&change.0) {
                                            senders.iter().for_each(|sender| {
                                                sender.send(change.1.clone()).unwrap();
                                            });
                                        }
                                    });
                                }

                                Subscribe::PatternEnvs(envs) => {
                                    let envs = envs
                                        .iter()
                                        .map(|pattern| Regex::new(pattern).unwrap())
                                        .collect::<Vec<Regex>>();

                                    changes.iter().for_each(|change| {
                                        envs.iter().for_each(|reg| {
                                            let mat = reg.find(&*change.0);
                                            match mat {
                                                None => {}
                                                Some(_) => {
                                                    senders.iter().for_each(|sender| {
                                                        sender.send(change.1.clone()).unwrap();
                                                    });
                                                }
                                            }
                                        });
                                    });
                                }
                            }
                        });
                    }
                };
                *data_guard = sys_data;
            }
            sleep(interval);
        });
    }
}

/// Default instance with read interval 30 seconds.
impl Default for EnvironmentWatcher {
    fn default() -> Self {
        let env_state = Self {
            state: Arc::new(Mutex::new(Default::default())),
            senders: Arc::new(Mutex::new(HashMap::default())),
            interval: Duration::from_millis(5 * 100),
        };
        env_state.run();
        env_state
    }
}
