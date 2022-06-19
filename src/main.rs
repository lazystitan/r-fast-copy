use std::fs;
use std::fs::{create_dir, create_dir_all, File, read_to_string};
use std::io::Write;
use std::path::{Path, PathBuf};

fn copy_file(from : &Path, to: &Path) {
    let content = fs::read_to_string(from).unwrap();
    let mut file = fs::File::create(to).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

fn copy_dir(to: &Path) {
    create_dir(to).unwrap();
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

fn vist_dirs(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        println!("parent is {}", entry.path().parent().unwrap().to_str().unwrap());
        if path.is_dir() {
            vist_dirs(&path);
        } else {
            println!("{}", entry.path().to_str().unwrap());
        }
    }
}

fn copy_dir_recursive(from: &Path, dest: &Path, depth_path: &PathBuf) {
    let read_dir = from.clone().to_path_buf().join(depth_path);
    println!("-----------");
    println!("from : {:?}", from);
    println!("dest : {:?}", dest);
    println!("depth_path : {:?}", depth_path);
    println!("read_dir : {:?}", read_dir);
    for entry in fs::read_dir(read_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let new_depth_path =
                depth_path.clone()
                    .join(Path::new(&entry.file_name()));
            println!("next depth's path: {:?}", new_depth_path);
            println!("creating path: {:?}", new_depth_path);
            let creating_path = dest.clone().to_path_buf().join(new_depth_path.clone());
            create_dir_all(&creating_path).unwrap();
            copy_dir_recursive(from, dest, &new_depth_path);
        } else {
            let new_depth_path =
                depth_path.clone()
                    .join(Path::new(&entry.file_name()));
            let creating_file = dest.clone().to_path_buf().join(new_depth_path.clone());
            let read_file = from.clone().to_path_buf().join(new_depth_path.clone());
            println!("creating file : {:?}", creating_file);
            let mut file = File::create(creating_file).unwrap();
            let content = read_to_string(read_file).unwrap();
            println!("copying content");
            file.write_all(content.as_bytes()).unwrap();
        }
    }
}

fn main() {
    copy_dir_recursive(Path::new("./test_dir/copy_test_dir"), Path::new("./test_dir/copy_test_dir_copied"), &PathBuf::new());
}
