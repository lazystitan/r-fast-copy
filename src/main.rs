mod test_gen;
mod copy;

use crate::test_gen::TestDirGenerator;
use clap::{Parser, Subcommand};
use std::fs::{create_dir_all};
use std::path::{Path};

#[derive(Subcommand, Debug)]
enum TestCommand {
    /// generate test folder in path
    GenTestFolder {
        #[clap(value_parser)]
        path: String,

        #[clap(value_parser, short = 'd', long)]
        max_depth: Option<u32>,

        #[clap(value_parser, short, long)]
        threshold: Option<f64>,
    },
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    // #[clap(short, long, value_parser)]
    // name: String,
    #[clap(short, long, value_parser, default_value_t = 1)]
    count: u8,

    #[clap(subcommand)]
    test_command: TestCommand,
}

fn main() {
    let args = Args::parse();
    for _ in 0..args.count {
        println!("Hello {}!", args.count);
    }
    match &args.test_command {
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
    // copy_dir_recursive(Path::new("./test_dir/copy_test_dir"), Path::new("./test_dir/copy_test_dir_copied"), &PathBuf::new());
}
