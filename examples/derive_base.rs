use std::env::set_var;
use std::time::Duration;
use env_watcher::{ChangeState, Error, Subscribe, sub_env, init_env_watch};

/// Basic version of the subscription, when you track the data yourself.
/// Analogous to [`examples/change_handler`]
fn main() -> Result<(), Error> {
    set_var("key.key", "vvv");

    init_env_watch!(Duration::from_millis(25 * 10))?;

    let sub = Subscribe::Envs(vec!["key.key".to_string()]);

    let (env, rx) = sub_env!(sub)?;

    assert_eq!(Some(&String::from("vvv")), env.get("key.key"));

    set_var("key.key", "hello");

    let event = rx.recv().unwrap();

    match event {
        ChangeState::Edit(k, v) => {
            let k = &*k;

            if k.eq("key.key") {
                assert_eq!("hello", &*v)
            }
        }
        ChangeState::Delete(_) => {}
    }

    Ok(())
}