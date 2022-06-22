use crate::pool::Message;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::{fs, io};

fn copy_file(from: &Path, to: &Path) -> Result<(), io::Error> {
    let content = fs::read_to_string(from)?;
    let mut file = fs::File::create(to)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn copy_dir_recursive(
    from: PathBuf,
    dest: PathBuf,
    depth_path: PathBuf,
    sender: Sender<Message>,
) -> Result<(), io::Error> {
    println!("-----------");
    println!("from : {:?}", from);
    println!("dest : {:?}", dest);
    println!("depth_path : {:?}", depth_path);
    let read_dir = from.clone().to_path_buf().join(depth_path.clone());
    println!("read_dir : {:?}", read_dir);
    for entry in fs::read_dir(read_dir)? {
        let entry = entry?;
        let path = entry.path();
        let new_depth_path = depth_path.clone().join(Path::new(&entry.file_name()));
        let creating_path = dest.clone().to_path_buf().join(new_depth_path.clone());
        if path.is_dir() {
            println!("next depth's path: {:?}", new_depth_path);
            println!("creating path: {:?}", new_depth_path);
            create_dir_all(&creating_path)?;

            // /*
            let new_from = from.clone();
            let new_dest = dest.clone();
            let new_new_depth_path = new_depth_path.clone();
            let new_sender = sender.clone();
            sender
                .send(Message::NewTask(Box::new(move || {
                    copy_dir_recursive(new_from, new_dest, new_new_depth_path, new_sender).unwrap();
                })))
                .unwrap();
            // */
        } else {
            let read_file = from.clone().to_path_buf().join(new_depth_path.clone());
            println!("creating file : {:?}", creating_path);
            copy_file(read_file.as_path(), creating_path.as_path())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod copy_test {
    use super::*;

    #[test]
    fn copy_file_test() {
        let from = Path::new("./test_dir/copy_test_dir/origin_file");
        let to = Path::new("./test_dir/copy_test_dir/copied_file1");
        copy_file(&from, &to).unwrap();
        let content_from = fs::read_to_string(&from).unwrap();
        let content_to = fs::read_to_string(&to).unwrap();
        assert_eq!(content_to, content_from);
    }
}
