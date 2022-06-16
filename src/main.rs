use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

static THRESHOLD: u32 = u32::MAX / 100 * 20;
static MAX_LEVEL: u32 = 5;

fn if_continue_gen(level: u32) -> bool {
    let i: u32 = thread_rng().gen();
    //the lesser the level is,  the return tends to true
    match i.checked_mul(MAX_LEVEL - level) {
        Some(r) => r > THRESHOLD,
        None => true,
    }
}

fn if_continue_gen_file() -> bool {
    let i: u32 = thread_rng().gen();
    i > THRESHOLD
}

fn gen_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn gen_dir_under_path(upper_path: &str, mut level: u32) {
    level += 1;
    let mut i = 0;

    while if_continue_gen_file() {
        println!("gen file");
        let file_name = gen_string(6);
        let upper = upper_path.to_string();
        let full_path = upper + "/" + &file_name + &i.to_string();
        let mut file = File::create(Path::new(&full_path)).unwrap();
        file.write_all(gen_string(100).as_bytes()).unwrap();
        i += 1;
    }

    if level >= MAX_LEVEL {
        return;
    }

    while if_continue_gen(level) {
        println!("gen dir");
        let dir_name = gen_string(6);
        let upper = upper_path.to_string();
        let full_path = upper + "/" + &dir_name + &i.to_string();
        let new_path_prefix = Path::new(&full_path);
        create_dir_all(new_path_prefix).unwrap();
        gen_dir_under_path(&full_path, level);
        i += 1;
    }
}

fn create_test_dir() {
    let origin_path = "./test_dir/origin";
    create_dir_all(Path::new(origin_path)).unwrap();
    gen_dir_under_path(origin_path, 0);
}

fn main() {
    create_test_dir();
}
