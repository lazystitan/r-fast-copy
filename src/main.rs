mod copy;
mod dir_tree;
mod pool;
mod test_gen;

use crate::copy::Copyer;
use crate::pool::ThreadPool;
use crate::test_gen::TestDirGenerator;
use clap::{Parser, Subcommand};
use std::fs::create_dir_all;
use std::path::Path;

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

    #[clap(short, long, value_parser, default_value_t = false)]
    single_thread: bool,

    // #[clap(short, long, value_parser)]
    // name: String,
    #[clap(subcommand)]
    test_command: Option<TestCommand>,
}

//threas    first time  second time average boost
//16	    14879       15123       15001 	51%
// 4	    16620       18419       17519.5	29%
// s	    21401       23953       22677	0%
fn main() {
    let args = Args::parse();
    if let Some(subcommand) = &args.test_command {
        subcommand.exec();
    }

    if let Some(from) = &args.from {
        println!("from: {}", from);
        match &args.to {
            Some(to) => {
                println!("to: {}", to);
                let mut builder = Copyer::builder().set_from(from).set_to(to);

                if !args.single_thread {
                    builder = builder
                        .set_multi_threads(!args.single_thread)
                        .set_threads_number(args.thread);
                }
                builder.build().unwrap().run();
            }
            None => {
                println!("Not set target");
            }
        }
    }
}
