mod test_gen;

use crate::test_gen::TestDirGenerator;
use clap::{Parser, Subcommand};
use std::fs;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn copy_file(from: &Path, to: &Path) {
    let content = fs::read_to_string(from).unwrap();
    let mut file = fs::File::create(to).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}
#[cfg(test)]
mod copy_test {
    use super::*;

    #[test]
    fn copy_file_test() {
        let from = Path::new("./test_dir/copy_test_dir/origin_file");
        let to = Path::new("./test_dir/copy_test_dir/copied_file1");
        copy_file(&from, &to);
        let content_from = fs::read_to_string(&from).unwrap();
        let content_to = fs::read_to_string(&to).unwrap();
        assert_eq!(content_to, content_from);
    }
}

fn copy_dir_recursive(from: &Path, dest: &Path, depth_path: &PathBuf) {
    println!("-----------");
    println!("from : {:?}", from);
    println!("dest : {:?}", dest);
    println!("depth_path : {:?}", depth_path);
    let read_dir = from.clone().to_path_buf().join(depth_path);
    println!("read_dir : {:?}", read_dir);
    for entry in fs::read_dir(read_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let new_depth_path = depth_path.clone().join(Path::new(&entry.file_name()));
        let creating_path = dest.clone().to_path_buf().join(new_depth_path.clone());
        if path.is_dir() {
            println!("next depth's path: {:?}", new_depth_path);
            println!("creating path: {:?}", new_depth_path);
            create_dir_all(&creating_path).unwrap();
            copy_dir_recursive(from, dest, &new_depth_path);
        } else {
            let read_file = from.clone().to_path_buf().join(new_depth_path.clone());
            println!("creating file : {:?}", creating_path);
            copy_file(read_file.as_path(), creating_path.as_path());
        }
    }
}

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
