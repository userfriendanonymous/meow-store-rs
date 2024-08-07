use std::fmt::Display;

use clap::Parser;
use tokio::{fs::{self, File}, io::AsyncWriteExt};

mod args;
mod db_config;
mod crawler_config;

// cargo run -- db create -c "../cli_usage/db_create.toml" -p "../cli_usage/db"
// cargo run -- db run -c "../cli_usage/db_run.toml" -p "../cli_usage/db"
// cargo run -- crawler run -c "../cli_usage/crawler.toml"

enum DbCommand {
    GenAuth,
}

impl Display for DbCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenAuth => f.write_str("generate auth key"),
        }
    }
}

#[tokio::main]
async fn main() {
    let args = args::Root::parse();

    match args.command {
        args::Sub::Db { command } => {
            match command {
                args::Db::Create { config, path } => {
                    fs::create_dir_all(&path).await.unwrap();
                    let config_str = fs::read_to_string(config).await.unwrap();
                    let config = toml::from_str::<db_config::Create>(&config_str).unwrap();
                    
                    fs::write(path.join("status"), "new".as_bytes()).await.unwrap();
                    fs::write(path.join("create.toml"), config_str.as_bytes()).await.unwrap();
                },
                args::Db::Run { config, path } => {
                    let config_str = fs::read_to_string(config).await.unwrap();
                    let config = toml::from_str::<db_config::Run>(&config_str).unwrap();
                    
                    let status = fs::read_to_string(path.join("status")).await.unwrap();
                    let create_config_str = fs::read_to_string(path.join("create.toml")).await.unwrap();
                    let create_config = toml::from_str::<db_config::Create>(&create_config_str).unwrap();

                    let mode = match status.as_str() {
                        "new" => db_http_server::OpenMode::New,
                        "existing" => db_http_server::OpenMode::Existing,
                        _ => panic!("Invalid status file. The database folder is corrupted.")
                    };
                    fs::write(path.join("status"), "existing".as_bytes()).await.unwrap();

                    println!("Running at {}", &config.addr);
                    let addr = config.addr.parse().unwrap();
                    let init = db_http_server::init_with_config(db_http_server::config::Run {
                        mode,
                        db: db::config::Root {
                            require_auth: config.require_auth,
                        },
                        db_path: path.join("db_data"),
                        addr,
                        meili_addr: config.meili_host,
                        meili_key: config.meili_key,
                    }).await;

                    {
                        let init = init.clone();
                        let _ = tokio::spawn(async move {
                            init.run().await
                        });
                    }

                    {
                        use inquire::{Text, Select, MultiSelect};
                        loop {
                            let options = vec![DbCommand::GenAuth];
                            match Select::new("Select a command:", options).prompt_skippable() {
                                Ok(Some(cmd)) => {
                                    match cmd {
                                        DbCommand::GenAuth => {
                                            use db::auth::Op;
                                            if let Ok(Some(ops)) = MultiSelect::new(
                                                "Select operations allowed with this key:",
                                                vec![Op::Read, Op::Write, Op::Remove]
                                            ).prompt_skippable() {
                                                let mut desc = db::auth::Desc::new_all_false();
                                                for op in ops {
                                                    match op {
                                                        Op::Read => desc.read = true,
                                                        Op::Write => desc.write = true,
                                                        Op::Remove => desc.remove = true,
                                                    }
                                                }
                                                match init.db.write().await.gen_auth(&desc).await {
                                                    Ok(key) => {
                                                        println!("Key:");
                                                        println!("{key}\n");
                                                    },
                                                    Err(e) => {
                                                        println!("Error: {:?}", e);
                                                    }
                                                }
                                            }
                                        },
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => println!("Error: {e}")
                            }
                        }
                    }
                }
            }
        },
        args::Sub::GenConfig { path } => {
            let db_create = db_config::Create {};
            let db_run = db_config::Run {
                addr: "127.0.0.1:3030".into(),
                meili_host: "http://localhost:7700".into(),
                meili_key: "aSampleMasterKey".into(),
                require_auth: db_config::RequireAuth {
                    read: false,
                    write: false,
                    remove: false,
                }
            };
            let crawler_run = crawler_config::Run {
                db_url: "http://localhost:3030".into(),
                initial_user: "griffpatch".into(),
                db_auth_key: None,
            };

            File::create_new(path.join("db_run.toml")).await.unwrap()
                .write_all(toml::to_string_pretty(&db_run).unwrap().as_bytes())
                .await.unwrap();

            File::create_new(path.join("db_create.toml")).await.unwrap()
                .write_all(toml::to_string_pretty(&db_create).unwrap().as_bytes())
                .await.unwrap();

            File::create_new(path.join("crawler.toml")).await.unwrap()
                .write_all(toml::to_string_pretty(&crawler_run).unwrap().as_bytes())
                .await.unwrap();
        },
        args::Sub::Crawler { command } => {
            match command {
                args::Crawler::Run { config } => {
                    let config_str = fs::read_to_string(config).await.unwrap();
                    let config = toml::from_str::<crawler_config::Run>(&config_str).unwrap();
                    crawler::run_with_config(crawler::config::Run {
                        db_url: config.db_url,
                        initial_user: config.initial_user,
                        db_auth_key: config.db_auth_key,
                    }).await;
                }
            }
        }
        // args::Root::Crawler(_) => {
        //     todo!()
        // }
    }
}
