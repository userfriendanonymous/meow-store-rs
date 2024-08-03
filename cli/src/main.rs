use clap::Parser;
use tokio::{fs::{self, File}, io::AsyncWriteExt};

mod args;
mod db_config;
mod crawler_config;

// cargo run -- db run -c "../cli_usage/db_run.toml" -p "../cli_usage/db"
// cargo run -- crawler run -c "../cli_usage/crawler.toml"

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
                    db_http_server::run_with_config(db_http_server::config::Run {
                        mode,
                        db_path: path.join("db_data"),
                        addr: config.addr.parse().unwrap(),
                        meili_addr: config.meili_host,
                        meili_key: config.meili_key,
                    }).await;
                }
            }
        },
        args::Sub::GenConfig { path } => {
            let db_create = db_config::Create {};
            let db_run = db_config::Run {
                addr: "127.0.0.1:3030".into(),
                meili_host: "http://localhost:7700".into(),
                meili_key: "aSampleMasterKey".into()
            };
            let crawler_run = crawler_config::Run {
                db_url: "http://localhost:3030".into(),
                initial_user: "griffpatch".into()
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
                    }).await;
                }
            }
        }
        // args::Root::Crawler(_) => {
        //     todo!()
        // }
    }
}
