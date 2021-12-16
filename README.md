[![ci](https://github.com/SinmoWay/env-watcher/actions/workflows/ci.yml/badge.svg)](https://github.com/SinmoWay/env-watcher/actions/workflows/ci.yml)

# A simple library for viewing environment variables with a subscription to change any variables.
Implementation of viewing variables in real time. In case of changing variables, get a snapshot of data or a change event (Delete/Edit).

# Usage

When used locally, you must create an instance of the observer that you will control. Duration - defines the polling time of variables. Default - `Duration::from_millis(5 * 100)`
```
let env_watcher = EnvironmentWatcher::new(Duration::from_secs(5));
```
Next, subscribe to any keys, all options are described in [paragraph](#Subscribing-to-environment-variables)

```
let sub_for_all = Subscribe::All;
let (data, rx) = env_watcher.subscribe(sub_for_all)?;
```
In this case, we received a snapshot of the data at the time of subscription, as well as the channel through which we will receive events.  
Suppose we have a running server that should restart if the `server.port` environment variable changes, let's try to write this!
```
fn listen_environment() -> Result<(), Error> {
    let env_watcher = EnvironmentWatcher::new(Duration::from_secs(5));
    
    // An analogue of this subscription can be: Subscribe::PatternEnvs(vec!["server.*".to_string()]); 
    let sub_server = Subscribe::Envs(vec!["server.host".to_string(), "server.port".to_string()]); 
    
    let (data, rx) = env_watcher.subscribe(sub_server)?;
    spawn(move || {
        let receiver = rx.clone();
        let data = data.clone();
        loop {
            let event = receiver.recv().unwrap();
            match event {
                ChangeState::Edit(k, v) => {
                    data.insert(k, v);
                    restart_server(&data);
                }
                ChangeState::Delete(k) => {
                    warn("Removed one of the server environment variables. In case of restarting the application, incorrect behavior may be expected!");
                }
            }
        }
    });
    Ok(())
}

fn restart_server(data: &HashMap<String, String>) {
    ...
    println("Restarting server");
}
```
You can see a more detailed example in the [project](examples/change_handler.rs).

# Base implementation for data

`EnvironmentData` serves as a basic snapshot keeper. In a separate thread, the values are updated if they change in the environment.

EnvironmentData serves as the primary storage for snapshots. On a separate thread, the values are updated if they change in the environment.

You can get a snapshot of the data using the `data()` method  

If a mutable reference is needed then use `ref_data()`  

# Derive usage
This module provides 3 macros to simplify your work.  
* `init_env_watch!` - basic storage initialization
* `sub_env!` - subscription with snapshot return and channel for event management
* `sub_env_snapshot!` - basic snapshot implementation

# Subscribing to environment variables

We have 3 subscription options in total.  
* All - subscribing to all changes to environment variables
* Envs - subscription for specific keys only
* PatternEnvs - subscribing only to specific keys using regular expressions thanks to the [library](https://docs.rs/regex/1.5.4/regex/)

# Release History

See [Changelog](CHANGELOG.md)

# License

MIT License