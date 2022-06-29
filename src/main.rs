mod copy;
mod dir_tree;
mod pool;
mod test_gen;

use crate::copy::Copyer;
use crate::pool::ThreadPool;
use crate::test_gen::TestDirGenerator;
use clap::{Parser, Subcommand};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// Generate test folder in path (relative to executable)
    GenTestFolder {
        #[clap(value_parser)]
        path: String,

        #[clap(value_parser, short = 'd', long)]
        max_depth: Option<u32>,

        #[clap(value_parser, long)]
        threshold: Option<f64>,
    },

    /// Benchmark, show copy cost time when threads number is 0(single-thread), 4, 8, 16, 32, 64
    /// and repeat 3 times. !!!Not for you to use.
    Benchmark
}

impl SubCommands {
    fn exec(&self) {
        match self {
            SubCommands::GenTestFolder {
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
            SubCommands::Benchmark => {
                let mut t_s = 0f64;
                let mut t_4 = 0f64;
                let mut t_8 = 0f64;
                let mut t_16 = 0f64;
                let mut t_32 = 0f64;
                let mut t_64 = 0f64;

                let from = "./test_dir/origin_2";
                // let to = "./test_dir/origin_2_cp";

                for i in 0..3 {
                    println!("-------{} loop start--------", i);
                    let builder = Copyer::builder().set_from(&from);

                    let now = Instant::now();
                    let to = format!("./test_dir/origin_2_cp_{}t_{}", 's', i);
                    let builder_t_s = builder.clone().set_to(&to);
                    builder_t_s.build().unwrap().run();
                    let elapsed_time = now.elapsed();
                    t_s += elapsed_time.as_millis() as f64;
                    println!("Single thread: {}", elapsed_time.as_millis());


                    let now = Instant::now();
                    let to = format!("./test_dir/origin_2_cp_{}t_{}", 4,i);
                    let builder_t_4 = builder.clone().set_to(&to);
                    builder_t_4.set_threads_number(4).build().unwrap().run();
                    let elapsed_time = now.elapsed();
                    t_4 += elapsed_time.as_millis() as f64;
                    println!("4 threads: {}", elapsed_time.as_millis());


                    let now = Instant::now();
                    let to = format!("./test_dir/origin_2_cp_{}t_{}", 8,i);
                    let builder_t_8 = builder.clone().set_to(&to);
                    builder_t_8.set_threads_number(4).build().unwrap().run();
                    let elapsed_time = now.elapsed();
                    t_8 += elapsed_time.as_millis() as f64;
                    println!("8 threads: {}", elapsed_time.as_millis());


                    let now = Instant::now();
                    let to = format!("./test_dir/origin_2_cp_{}t_{}", 16,i);
                    let builder_t_16 = builder.clone().set_to(&to);
                    builder_t_16.set_threads_number(4).build().unwrap().run();
                    let elapsed_time = now.elapsed();
                    t_16 += elapsed_time.as_millis() as f64;
                    println!("16 threads: {}", elapsed_time.as_millis());


                    let now = Instant::now();
                    let to = format!("./test_dir/origin_2_cp_{}t_{}", 32,i);
                    let builder_t_32 = builder.clone().set_to(&to);
                    builder_t_32.set_threads_number(4).build().unwrap().run();
                    let elapsed_time = now.elapsed();
                    t_32 += elapsed_time.as_millis() as f64;
                    println!("32 threads: {}", elapsed_time.as_millis());


                    let now = Instant::now();
                    let to = format!("./test_dir/origin_2_cp_{}t_{}", 64,i);
                    let builder_t_64 = builder.clone().set_to(&to);
                    builder_t_64.set_threads_number(4).build().unwrap().run();
                    let elapsed_time = now.elapsed();
                    t_64 += elapsed_time.as_millis() as f64;
                    println!("64 threads: {}", elapsed_time.as_millis());
                }

                println!("single threads average time {}", (t_s / 3f64) as i32);
                println!("4 threads average time {}", (t_4 / 3f64) as i32);
                println!("8 threads average time {}", (t_8 / 3f64) as i32);
                println!("16 threads average time {}", (t_16 / 3f64) as i32);
                println!("32 threads average time {}", (t_32 / 3f64) as i32);
                println!("64 threads average time {}", (t_64 / 3f64) as i32);


            }
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    ///Path to copy
    #[clap(value_parser)]
    from: Option<String>,

    ///Destination
    #[clap(value_parser)]
    to: Option<String>,

    ///Multi-threads mode threads number
    #[clap(short, long, value_parser, default_value_t = 4)]
    thread: usize,

    ///Single-threads mode
    #[clap(short, long, value_parser, default_value_t = false)]
    single_thread: bool,

    #[clap(subcommand)]
    sub: Option<SubCommands>,

    ///Details in copy action
    #[clap(short, long, value_parser, default_value_t = false)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    if let Some(subcommand) = &args.sub {
        subcommand.exec();
    }

    if args.from.is_some() && args.to.is_some() {
        let from = &args.from.unwrap();
        println!("from: {}", from);
        let to = &args.to.unwrap();
        println!("to: {}", to);
        let mut builder = Copyer::builder().set_from(from).set_to(to).set_verbose(args.verbose);

        if !args.single_thread {
            builder = builder.set_threads_number(args.thread);
        }
        builder.build().unwrap().run();
    } else if (args.from.is_some() || args.to.is_some()) && !(args.from.is_none() && args.to.is_none()) {
        println!("Not set target or from path.");
    }
}
