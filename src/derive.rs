use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use crossbeam_channel::Receiver;
use state::Storage;
use crate::{ChangeState, EnvironmentData, EnvironmentWatcher, Subscribe, Result, Error};

static ENV_WATCHER: Storage<Mutex<EnvironmentWatcher>> = Storage::new();
static INIT: Storage<Mutex<i8>> = Storage::new();

/// Initialize env watcher (singleton) with duration.
#[macro_export]
macro_rules! init_env_watch {
    ($dur:expr) => {
        {
            $crate::derive::init_env_watcher($dur)
        }
    }
}

/// Subscribe with Receiver and Data
#[macro_export]
macro_rules! sub_env {
    ($sub:expr) => {
        {
            $crate::derive::subscribe($sub)
        }
    }
}

/// Subscribe by snapshot
#[macro_export]
macro_rules! sub_env_snapshot {
    ($sub:expr) => {
        {
            $crate::derive::subscribe_snapshot($sub)
        }
    }
}

#[doc(hidden)]
pub fn init_env_watcher(interval: Duration) -> Result<()> {
    let mut init = INIT.get_or_set(|| Mutex::new(0)).lock().unwrap();

    if *init > 0 {
        return Err(Error::DoubleInitialWatcher);
    }

    let watcher = ENV_WATCHER.get_or_set(|| Mutex::new(EnvironmentWatcher::new(interval))).lock().unwrap();

    if watcher.size() > 0 {
        return Err(Error::ReinitializedWithSubscribers);
    }

    *init = 1;

    Ok(())
}

#[doc(hidden)]
pub fn subscribe(sub: Subscribe) -> Result<(HashMap<String, String>, Receiver<ChangeState>)> {
    let watcher = ENV_WATCHER.get().lock().unwrap();
    watcher.subscribe(sub)
}

#[doc(hidden)]
pub fn subscribe_snapshot(sub: Subscribe) -> Result<EnvironmentData> {
    let watcher = ENV_WATCHER.get().lock().unwrap();
    watcher.subscribe_snapshot(sub)
}