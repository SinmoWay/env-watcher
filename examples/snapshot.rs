use env_watcher::{EnvironmentWatcher, Error, Subscribe};
use std::thread::sleep;
use std::time::Duration;

static TEST_VALUE: &'static str = "ONLY_TEST";
static TEST_BY_CHANGE_VALUE: &'static str = "ONLY_TEST_CHANGE";

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
    let sub_envs_key = vec!["test.bey".to_string(), "test.wow".to_string()];
    fill_envs(sub_envs_key.clone(), TEST_VALUE);

    let sub_envs = Subscribe::Envs(sub_envs_key.clone());

    let sub_pattern_envs_key = vec![
        "my.server.port".to_string(),
        "my.server.host".to_string(),
        "my.server.locale".to_string(),
        "my.server.tls".to_string(),
    ];
    fill_envs(sub_pattern_envs_key.clone(), TEST_VALUE);

    let sub_pattern_envs = Subscribe::PatternEnvs(vec!["^my.server.*".to_string()]);

    let env_core = EnvironmentWatcher::new(Duration::from_secs(3));
    sleep(Duration::from_secs(1));

    let only_envs = env_core.subscribe_snapshot(sub_envs)?;

    let envs_data = only_envs.data();
    print_after();
    println!("{:?}", &envs_data);
    println!();

    envs_data.iter().for_each(|kv| {
        match &**kv.0 {
            "test.bey" | "test.wow" => {
                // All good.
                assert_eq!(TEST_VALUE, &**kv.1);
            }
            _ => {
                println!("Find unknown key with value. {}, {}", &**kv.0, &**kv.1);
                panic!("unknown k/v in pattern sub.");
            }
        }
    });

    let pattern_envs = env_core.subscribe_snapshot(sub_pattern_envs)?;

    let pattern_data = pattern_envs.data();
    println!("{:?}", &pattern_data);

    pattern_data.iter().for_each(|kv| {
        match &**kv.0 {
            "my.server.port" | "my.server.host" | "my.server.locale" | "my.server.tls" => {
                // All good.
                assert_eq!(TEST_VALUE, &**kv.1);
            }
            _ => {
                println!("Find unknown key with value. {}, {}", &**kv.0, &**kv.1);
                panic!("unknown k/v in pattern sub.");
            }
        }
    });

    fill_envs(sub_envs_key.clone(), TEST_BY_CHANGE_VALUE);
    fill_envs(sub_pattern_envs_key.clone(), TEST_BY_CHANGE_VALUE);

    sleep(Duration::from_secs(3));

    print_before();
    let envs_data = only_envs.data();
    println!("{:?}", &envs_data);
    println!();

    envs_data.iter().for_each(|kv| {
        match &**kv.0 {
            "test.bey" | "test.wow" => {
                // All good.
                assert_eq!(TEST_BY_CHANGE_VALUE, &**kv.1);
            }
            _ => {
                println!("Find unknown key with value. {}, {}", &**kv.0, &**kv.1);
                panic!("unknown k/v in pattern sub.");
            }
        }
    });

    let pattern_data = pattern_envs.data();
    println!("{:?}", &pattern_data);

    pattern_data.iter().for_each(|kv| {
        match &**kv.0 {
            "my.server.port" | "my.server.host" | "my.server.locale" | "my.server.tls" => {
                // All good.
                assert_eq!(TEST_BY_CHANGE_VALUE, &**kv.1);
            }
            _ => {
                println!("Find unknown key with value. {}, {}", &**kv.0, &**kv.1);
                panic!("unknown k/v in pattern sub.");
            }
        }
    });

    Ok(())
}
