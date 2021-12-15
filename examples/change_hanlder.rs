use env_watcher::{ChangeState, EnvironmentWatcher, Error, Subscribe};
use std::collections::HashMap;
use std::env::{remove_var, set_var};
use std::thread::sleep;
use std::time::Duration;

static TEST_VALUE: &'static str = "ONLY_TEST";

fn fill_envs(envs: Vec<String>, val: &'static str) {
    envs.iter().for_each(|k| {
        std::env::set_var(k, &val);
    });
}

fn print_after() {
    println!();
    println!("###################################################");
    println!("############## SUB KEYS AFTER CHANGE ##############");
    println!("###################################################");
    println!();
}

fn print_before() {
    println!();
    println!("####################################################");
    println!("############## SUB KEYS BEFORE CHANGE ##############");
    println!("####################################################");
    println!();
}

pub fn main() -> Result<(), Error> {
    let sub_envs_key = vec![
        "server.port".to_string(),
        "server.address".to_string(),
        "server.tls".to_string(),
    ];
    fill_envs(sub_envs_key.clone(), TEST_VALUE);

    let env_core = EnvironmentWatcher::default();
    sleep(Duration::from_secs(1));

    let sub_envs = Subscribe::Envs(sub_envs_key.clone());
    let only_envs = env_core.subscribe(sub_envs)?;

    print_after();

    let mut data = only_envs.0;

    println!("{:?}", &data);

    let rec = only_envs.1;

    let mut event_count = 0;
    loop {
        // Change env state.
        if event_count == 0 {
            set_var("server.port", "2013");
        } else if event_count == 1 {
            set_var("server.address", "localhost");
        } else if event_count == 2 {
            remove_var("server.tls");
        }

        let event = rec.recv().unwrap();

        match event {
            ChangeState::Edit(k, v) => {
                match &*k {
                    "server.port" => {
                        assert_eq!("2013", &*v);
                        data.insert(k, v);
                        restart_server(&data);
                    }
                    "server.address" => {
                        assert_eq!("localhost", &*v);
                        data.insert(k, v);
                        restart_server(&data);
                    }
                    _ => {
                        // ignore
                    }
                }
            }
            ChangeState::Delete(k) => {
                assert_eq!("server.tls", &*k);
                data.remove(&k);
                restart_server(&data);
            }
        }

        event_count = event_count + 1;

        if event_count >= 3 {
            break;
        }
    }

    Ok(())
}

fn restart_server(data: &HashMap<String, String>) {
    print_before();
    println!("{:?}", data);

    println!("Server has been restart.")
}
