use std::env::set_var;
use std::thread::sleep;
use crate::{ChangeState, EnvironmentWatcher, Subscribe, init_env_watch, sub_env, sub_env_snapshot};
use regex::Regex;
use std::time::Duration;

static TEST_VALUE: &'static str = "ONLY_TEST";

fn fill_envs(envs: Vec<String>) {
    envs.iter().for_each(|k| {
        std::env::set_var(k, &TEST_VALUE);
    });
}

#[test]
pub fn create_all_subscriber() {
    let subscribe = Subscribe::All;
    let vec_envs = vec!["my.test34.host", "my.test34.port", "my.test34.type"]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    fill_envs(vec_envs.clone());

    let env_watcher = EnvironmentWatcher::new(Duration::from_secs(5));
    spin_sleep::sleep(Duration::from_secs(2));
    let data = env_watcher.subscribe(subscribe).unwrap();

    let current_data = data.0;

    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.test34.host").map(|v| &**v)
    );
    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.test34.port").map(|v| &**v)
    );
    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.test34.type").map(|v| &**v)
    );

    let tx = data.1;

    std::env::remove_var("my.test34.host");
    std::env::set_var("my.test34.port", "2011");
    std::env::set_var("my.test34.type", "test");

    let mut i = 0;
    loop {
        let state = tx.recv().unwrap();
        match state {
            ChangeState::Edit(k, v) => {
                println!("Change state. Edit: key - {}, value - {}", &*k, &*v);
                let key = &*k;
                match key {
                    "my.test34.port" => {
                        assert_eq!("2011", &*v);
                        i = i + 1;
                    }
                    "my.test34.type" => {
                        assert_eq!("test", &*v);
                        i = i + 1;
                    }
                    _ => {
                        // Ignore
                    }
                };
            }
            ChangeState::Delete(k) => {
                println!("Change state. Delete: key - {}", &*k);
                if k.eq("my.test34.host") {
                    i = i + 1;
                }
            }
        }
        if i >= 3 {
            break;
        }
    }
}

#[test]
pub fn create_envs_subscriber() {
    let vec_envs = vec!["my.test.host", "my.test.port", "my.test.type"]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    fill_envs(vec_envs.clone());
    let subscribe = Subscribe::Envs(vec_envs.clone());

    let env_watcher = EnvironmentWatcher::new(Duration::from_secs(5));
    spin_sleep::sleep(Duration::from_secs(2));
    let data = env_watcher.subscribe(subscribe).unwrap();

    let current_data = data.0;
    println!("{:?}", &current_data);

    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.test.host").map(|v| &**v)
    );
    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.test.port").map(|v| &**v)
    );
    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.test.type").map(|v| &**v)
    );

    let tx = data.1;

    std::env::remove_var("my.test.host");
    std::env::set_var("my.test.port", "2011");
    std::env::set_var("my.test.type", "test");

    let mut i = 0;
    loop {
        let state = tx.recv().unwrap();
        match state {
            ChangeState::Edit(k, v) => {
                println!("Change state. Edit: key - {}, value - {}", &*k, &*v);
                let key = &*k;
                match key {
                    "my.test.port" => {
                        assert_eq!("2011", &*v);
                        i = i + 1;
                    }
                    "my.test.type" => {
                        assert_eq!("test", &*v);
                        i = i + 1;
                    }
                    _ => {
                        // Ignore
                    }
                };
            }
            ChangeState::Delete(k) => {
                println!("Change state. Delete: key - {}", &*k);
                if k.eq("my.test.host") {
                    i = i + 1;
                }
            }
        }
        if i >= 3 {
            break;
        }
    }
}

#[test]
pub fn create_pattern_envs_subscriber() {
    let vec_second_envs = vec!["my.client.host", "my.client.port", "my.client.blob.size"]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    fill_envs(vec_second_envs.clone());

    let subscriber = Subscribe::PatternEnvs(vec!["my.client.*".to_string()]);

    let env_watcher = EnvironmentWatcher::new(Duration::from_secs(5));
    spin_sleep::sleep(Duration::from_secs(2));
    let data = env_watcher.subscribe(subscriber).unwrap();

    let current_data = data.0;
    println!("{:?}", &current_data);

    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.client.host").map(|v| &**v)
    );
    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.client.port").map(|v| &**v)
    );
    assert_eq!(
        Some(TEST_VALUE),
        current_data.get("my.client.blob.size").map(|v| &**v)
    );

    let tx = data.1;

    std::env::remove_var("my.client.host");
    std::env::set_var("my.client.port", "2011");
    std::env::set_var("my.client.blob.size", "7MB");

    let mut i = 0;
    loop {
        let state = tx.recv().unwrap();
        match state {
            ChangeState::Edit(k, v) => {
                println!("Change state. Edit: key - {}, value - {}", &*k, &*v);
                let key = &*k;
                match key {
                    "my.client.port" => {
                        assert_eq!("2011", &*v);
                        i = i + 1;
                    }
                    "my.client.blob.size" => {
                        assert_eq!("7MB", &*v);
                        i = i + 1;
                    }
                    _ => {
                        // Ignore
                    }
                };
            }
            ChangeState::Delete(k) => {
                println!("Change state. Delete: key - {}", &*k);
                if k.eq("my.client.host") {
                    i = i + 1;
                }
            }
        }
        if i >= 3 {
            break;
        }
    }
}

#[test]
pub fn snapshot_changes() {
    let subscribe = Subscribe::All;
    let vec_envs = vec!["my.test44.host", "my.test44.port", "my.test44.type"]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    fill_envs(vec_envs.clone());

    let env_watcher = EnvironmentWatcher::new(Duration::from_secs(5));
    spin_sleep::sleep(Duration::from_secs(2));
    let data = env_watcher.subscribe_snapshot(subscribe).unwrap();

    let w_data = data.data();

    assert_eq!(Some(TEST_VALUE), w_data.get("my.test.host").map(|v| &**v));
    assert_eq!(Some(TEST_VALUE), w_data.get("my.test.port").map(|v| &**v));
    assert_eq!(Some(TEST_VALUE), w_data.get("my.test.type").map(|v| &**v));

    std::env::remove_var("my.test44.host");
    std::env::set_var("my.test44.port", "2012");
    std::env::set_var("my.test44.type", "78MB");

    spin_sleep::sleep(Duration::from_secs(6));

    let data = data.data();
    println!("{:?}", &data);

    assert_eq!(Some("2012"), data.get("my.test44.port").map(|v| &**v));
    assert_eq!(Some("78MB"), data.get("my.test44.type").map(|v| &**v));
    assert_eq!(None, data.get("my.test44.host"));
}

#[test]
pub fn find_sub() {
    let my_str = "my.client.host";

    let my_pattern = "^my.client.*";

    let regex = Regex::new(my_pattern).unwrap();

    let r = regex.find(my_str);

    assert!(r.is_some());

    let res = r.unwrap();
    assert_eq!("my.client.host", res.as_str());
}

#[test]
pub fn find_sub2() {
    let my_str = "my.client.host.version.1";

    let my_pattern = "^*.host.*";

    let regex = Regex::new(my_pattern).unwrap();

    let r = regex.find(my_str);

    assert!(r.is_some());
    assert!(regex.find("my.not.found.test").is_none());

    let res = r.unwrap();
    println!("{}", res.as_str())
}

#[test]
pub fn test_init_with_sub() -> crate::Result<()> {
    set_var("test.west.key", "hello world");
    set_var("test.west.key2", "hello world2");
    init_env_watch!(Duration::from_millis(500))?;

    sleep(Duration::from_millis(600));
    let sub = Subscribe::Envs(vec!["test.west.key".to_string(), "test.west.key2".to_string()]);
    let (data, rx) = sub_env!(sub.clone())?;
    let snap = sub_env_snapshot!(sub)?;

    let west_key = data.get("test.west.key").map(|v| &**v);
    let west_key2 = data.get("test.west.key2").map(|v| &**v);

    match west_key {
        None => {
            panic!("Not found key test.west.key");
        }
        Some(v) => {
            assert_eq!(v, "hello world")
        }
    }

    match west_key2 {
        None => {
            panic!("Not found key test.west.key2");
        }
        Some(v) => {
            assert_eq!(v, "hello world2")
        }
    }

    let mut d = snap.data();

    assert!(d.eq(&data));

    set_var("test.west.key", "derive");

    match rx.recv() {
        Ok(state) => {
            match state {
                ChangeState::Edit(k, v) => {
                    match &*k {
                        "test.west.key" => {
                            d.insert(k.clone(), v.clone());
                            assert_eq!(&*v, "derive")
                        }
                        _ => {}
                    }
                }
                ChangeState::Delete(_) => {}
            }
        }
        Err(e) => {
            panic!("Receiver error. {:?}", e)
        }
    }

    println!("Current data in rx: {:?}", &d);
    println!("Current data in snap: {:?}", &snap.data());

    let mut x = 0;
    loop {
        if d.eq(&snap.data()) {
            println!("Awaiting is {}millis", x * 100);
            break;
        } else if x >= 15 {
            panic!("Awaiting data return err. Max attempt exceeded.");
        }
        x = x + 1;
        sleep(Duration::from_millis(100));
    }

    Ok(())
}