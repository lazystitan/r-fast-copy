mod copy;
mod pool;
mod test_gen;
mod dir_tree;

use std::cell::RefCell;
use std::collections::HashMap;
use crate::copy::copy_dir_recursive;
use crate::test_gen::TestDirGenerator;
use clap::{Parser, Subcommand};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};
use rand::rngs::ThreadRng;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use crate::pool::{Message, ThreadPool};

#[derive(Subcommand, Debug)]
enum TestCommand {
    /// Generate test folder in path
    GenTestFolder {
        #[clap(value_parser)]
        path: String,

        #[clap(value_parser, short = 'd', long)]
        max_depth: Option<u32>,

        #[clap(value_parser, long)]
        threshold: Option<f64>,
    },
}

impl TestCommand {
    fn exec(&self) {
        match self {
            TestCommand::GenTestFolder {
                path,
                max_depth,
                threshold,
            } => {
                let path = Path::new(path);
                println!(
                    "path: {:?}, exists: {:?}, is_dir: {:?}",
                    path,
                    path.exists(),
                    path.is_dir()
                );

                if !path.exists() {
                    create_dir_all(path).unwrap();
                }

                let mut g = TestDirGenerator::builder();
                g = g.upper_path(path);

                if let Some(md) = max_depth {
                    g = g.max_depth(*md);
                }

                if let Some(t) = threshold {
                    g = g.threshold(*t);
                }

                g.build().unwrap().gen().unwrap()
            }
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(value_parser)]
    from: Option<String>,

    #[clap(value_parser)]
    to: Option<String>,

    #[clap(short, long, value_parser, default_value_t = 4)]
    thread: usize,

    // #[clap(short, long, value_parser)]
    // name: String,

    #[clap(subcommand)]
    test_command: Option<TestCommand>,
}

fn run() {
    let args = Args::parse();
    if let Some(subcommand) = &args.test_command {
        subcommand.exec();
    }

    if let Some(from) = &args.from {
        println!("from: {}", from);
        match &args.to {
            Some(to) => {
                println!("to: {}", to);

                let pool = ThreadPool::new(args.thread);
                let sender = pool.sender.clone();
                let now = Instant::now();
                copy_dir_recursive(PathBuf::from(from), PathBuf::from(to), PathBuf::new(), sender)
                    .expect("Copy failed");
                let elapsed_time = now.elapsed();
                println!("Copy action took {} milliseconds", elapsed_time.as_millis());
            }
            None => {
                println!("Not set target");
            }
        }
    }
}

fn main() {
    run();
}
