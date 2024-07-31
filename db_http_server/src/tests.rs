use std::borrow::Cow;
use std::str::FromStr;
use rand::seq::SliceRandom;
use rand::Rng;


// #[test]
pub async fn test1() {
    let mut rng = rand::thread_rng();
    let mut db = unsafe { db::Value::open("test_db", db::OpenMode::New) }.unwrap();

    for _ in 0 .. 30 {
        let mut names = Vec::new();

        let t = std::time::Instant::now();
        for i in 0 .. 100 {
            let len = rng.gen_range(1u8 ..= 20);
            let mut content = [0u8; 20];
            for idx in 0 .. len {
                content[idx as usize] = rng.gen_range(0 .. db::Username::CHARS.len() as u8);
            }
            let name = unsafe { db::Username::from_raw(len, content) };

            use rand::distributions::{Alphanumeric, DistString};

            let len = rng.gen_range(0 .. 20);
            let status = Alphanumeric.sample_string(&mut rng, len);

            let user = db::user::Value {
                name: name.clone(),
                id: 55555,
                scratch_team: false,
                status: Cow::Owned(status),
                bio: Cow::Borrowed("Amazing bio!")
            };

            if !db.add_user(user).await.unwrap() {
                names.push(name.clone());
            }
        }
        println!("write ms: {}", t.elapsed().as_millis());

        let t = std::time::Instant::now();
        for name in &names {
            let user = db.user_by_name(name).unwrap();
            assert_eq!(&user.name, name);
        }
        println!("get ms: {}", t.elapsed().as_millis());

        let t = std::time::Instant::now();
        for name in &names {
            if db.remove_user_by_name(name).unwrap() {
                panic!("Name didn't exist!");
            }
        }
        println!("Remove free locations: {}", db.users.free_locations_len());
        println!("remove ms: {}", t.elapsed().as_millis());

        for name in &names {
            assert!(db.user_by_name(name).is_none());
        }
    }
}