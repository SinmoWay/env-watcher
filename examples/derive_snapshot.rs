use std::env::set_var;
use std::thread::sleep;
use std::time::Duration;
use env_watcher::{Error, Subscribe, sub_env_snapshot, init_env_watch};

/// In this case, we get a reference to the values, when the environment variable changes, there is a change in the reference.
/// Analogous to [`examples/snapshot.rs`]
fn main() -> Result<(), Error> {
    set_var("key.key", "vvv");

    init_env_watch!(Duration::from_millis(25 * 10))?;

    let sub = Subscribe::Envs(vec!["key.key".to_string()]);

    let env = sub_env_snapshot!(sub)?;

    assert_eq!(Some(&String::from("vvv")), env.data().get("key.key"));

    set_var("key.key", "hello");

    sleep(Duration::from_millis(500));

    assert_eq!(Some(&String::from("hello")), env.data().get("key.key"));

    Ok(())
}