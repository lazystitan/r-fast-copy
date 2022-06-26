use crate::dir_tree::{DirNode, SharedNodeRef};
use crate::pool::Message;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::{fs, io};

// TODO May causing problem when handle really big file
fn copy_file(from: &Path, to: &Path) -> Result<(), io::Error> {
    let content = fs::read_to_string(from)?;
    let mut file = fs::File::create(to)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn copy_dir_recursive_single_thread(
    from: &Path,
    dest: &Path,
    depth_path: &PathBuf,
) -> Result<(), io::Error> {
    println!("-----------");
    println!("from : {:?}", from);
    println!("dest : {:?}", dest);
    println!("depth_path : {:?}", depth_path);
    let read_dir = from.clone().to_path_buf().join(depth_path);
    println!("read_dir : {:?}", read_dir);
    for entry in fs::read_dir(read_dir)? {
        let entry = entry?;
        let path = entry.path();
        let new_depth_path = depth_path.clone().join(Path::new(&entry.file_name()));
        let creating_path = dest.clone().to_path_buf().join(new_depth_path.clone());
        if path.is_dir() && !path.is_symlink() {
            println!("next depth's path: {:?}", new_depth_path);
            println!("creating path: {:?}", new_depth_path);
            create_dir_all(&creating_path)?;
            copy_dir_recursive_single_thread(from, dest, &new_depth_path)?;
        } else if path.is_file() || path.is_symlink() {
            let read_file = from.clone().to_path_buf().join(new_depth_path.clone());
            println!("creating file : {:?}", creating_path);
            copy_file(read_file.as_path(), creating_path.as_path())?;
        }
    }

    Ok(())
}

pub fn copy_dir_recursive(
    from: PathBuf,
    dest: PathBuf,
    depth_path: PathBuf,
    sender: Sender<Message>,
    parent_node: SharedNodeRef,
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
        if path.is_dir() && !path.is_symlink() {
            println!("next depth's path: {:?}", new_depth_path);
            println!("creating path: {:?}", new_depth_path);
            create_dir_all(&creating_path)?;

            // Create new node for directory in this loop, and then attach it to directory tree and
            // set parent for it.
            println!("creating new node for path {:?}", creating_path);
            let mut node = DirNode::new(creating_path);
            node.set_parent(parent_node.clone());
            let node_r = SharedNodeRef::new(node);

            println!("add new node to parent");
            let mut writer = parent_node.inner().write().unwrap();
            writer.add_sub_nodes(node_r.clone());
            drop(writer);
            println!("attach node to tree done");

            let new_from = from.clone();
            let new_dest = dest.clone();
            let new_new_depth_path = new_depth_path.clone();
            let new_sender = sender.clone();

            //For directory under this directory, make it as a new task to pool.
            sender
                .send(Message::NewTask(Box::new(move || {
                    copy_dir_recursive(new_from, new_dest, new_new_depth_path, new_sender, node_r)
                        .unwrap();
                })))
                .unwrap();
        } else if path.is_file() || path.is_symlink() {
            let read_file = from.clone().to_path_buf().join(new_depth_path.clone());
            println!("creating file : {:?}", creating_path);
            copy_file(read_file.as_path(), creating_path.as_path())?;
        }
    }

    let mut writer = parent_node.inner().write().unwrap();
    println!("start lookup {:?}", writer.path());
    writer.set_copied(); //当前node的父node检查
    drop(writer);

    try_lookup_continuously(parent_node);

    Ok(())
}

// If current node is copied and has parent, set current node to parent and repeat.
fn try_lookup_continuously(start_node: SharedNodeRef) {
    let reader = start_node.inner().read().unwrap();
    let mut lookup_flag = reader.is_copied();

    let mut may_parent = None;

    if let Some(p) = &reader.parent() {
        may_parent = Some(p.clone());
    }
    drop(reader);

    while lookup_flag && may_parent.is_some() {
        let parent = may_parent.take().unwrap();
        let mut writer = parent.inner().write().unwrap();
        writer.set_copied();
        drop(writer);

        let reader = parent.inner().read().unwrap();
        lookup_flag = reader.is_copied();

        if let Some(p) = &reader.parent() {
            may_parent = Some(p.clone());
        }
    }
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
