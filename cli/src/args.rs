
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Root {
    #[clap(subcommand)]
    pub command: Sub,
}

#[derive(Subcommand, Debug)]
#[command(version, about, long_about = None)]
pub enum Sub {
    Db {
        #[clap(subcommand)]
        command: Db
    },
    GenConfig {
        #[arg(long, short)]
        path: PathBuf,
    },
}

// meowstore db create -c db_create.toml -p ./db
// meowstore db run -c db_run.toml -p ./db
#[derive(Subcommand, Debug)]
#[command(version, about, long_about = None)]
pub enum Db {
    Create {
        #[arg(long, short)]
        config: PathBuf,
        #[arg(long, short)]
        path: PathBuf,
    },
    Run {
        #[arg(long, short)]
        config: PathBuf,
        #[arg(long, short)]
        path: PathBuf,
    }
}

// #[derive(Args, Debug)]
// #[command(version, about, long_about = None)]
// pub enum Crawler {

// }

