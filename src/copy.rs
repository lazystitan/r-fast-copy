use crate::dir_tree::{DirNode, SharedNodeRef};
use crate::pool::Message;
use crate::ThreadPool;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{fs, io, thread};

#[derive(Clone)]
pub struct CopyBuilder {
    verbose: bool,
    multi_threads: bool,
    threads_number: usize,
    from: Option<String>,
    to: Option<String>,
}

impl CopyBuilder {
    pub fn set_threads_number(mut self, num: usize) -> Self {
        self.threads_number = num;
        self.multi_threads = true;
        self
    }

    pub fn set_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn set_from(mut self, from: &str) -> Self {
        self.from = Some(String::from(from));
        self
    }

    pub fn set_to(mut self, to: &str) -> Self {
        self.to = Some(String::from(to));
        self
    }

    fn parse_from(from: &str) -> Result<String, &'static str> {
        let abs_from = fs::canonicalize(Path::new(from));
        if abs_from.is_err() {
            return Err("Preprocess from param failed.");
        }
        return Ok(String::from(abs_from.unwrap().to_str().unwrap()));
    }

    fn parse_to(to: &str) -> Result<String, &'static str> {
        let mut abs_to_pathbuff = PathBuf::from(to);
        let abs_to = fs::canonicalize(Path::new(to));

        if abs_to.is_err() {
            let mut count = 0;
            while !abs_to_pathbuff.exists() {
                let p = abs_to_pathbuff.pop();
                if !p {
                    return Err("Preprocess to param failed");
                }
                count += 1;
            }
            let abs_to_pathbuff_children = PathBuf::from(to);
            let mut new_buff = PathBuf::new();
            for (i, path) in abs_to_pathbuff_children.iter().rev().enumerate() {
                new_buff.push(path);
                if i >= count - 1 {
                    break;
                }
            }

            let children = new_buff.iter().rev().collect::<PathBuf>();
            abs_to_pathbuff = fs::canonicalize(abs_to_pathbuff).unwrap();
            abs_to_pathbuff = abs_to_pathbuff.join(children);
        }

        if create_dir_all(abs_to_pathbuff.clone()).is_err() {
            return Err("Create to directory failed");
        }

        return Ok(String::from(abs_to_pathbuff.to_str().unwrap()));
    }

    fn path_preprocess(from: &str, to: &str) -> Result<(String, String), &'static str> {
        let from = Self::parse_from(from);
        if let Err(e) = from {
            return Err(e);
        }
        let to = Self::parse_to(to);
        if let Err(e) = to {
            return Err(e);
        }

        return Ok((from.unwrap(), to.unwrap()));
    }

    pub fn build(self) -> Result<Copyer, &'static str> {
        let r = Self::path_preprocess(&self.from.unwrap(), &self.to.unwrap());
        if r.is_err() {
            return Err("Wrong.");
        }

        let (abs_from, abs_to) = r.unwrap();
        let mut pool = None;
        let mut root = None;
        if self.threads_number > 0 {
            pool = Some(ThreadPool::new(self.threads_number));
            root = Some(SharedNodeRef::new(DirNode::new(PathBuf::from(abs_to.clone()), self.verbose)));
        }

        Ok(Copyer {
            verbose: self.verbose,
            multi_threads: self.multi_threads,
            pool,
            from: PathBuf::from(abs_from),
            to: PathBuf::from(abs_to),
            _root: root,
        })
    }
}

pub struct Copyer {
    verbose: bool,
    multi_threads: bool,
    pool: Option<ThreadPool>,
    from: PathBuf,
    to: PathBuf,
    _root: Option<SharedNodeRef>,
}

impl Copyer {
    pub fn builder() -> CopyBuilder {
        CopyBuilder {
            verbose: false,
            multi_threads: false,
            threads_number: 0,
            from: None,
            to: None,
        }
    }

    pub fn run_multi_threads(self) {
        let pool_ref = self.pool.as_ref().unwrap();
        Self::copy_dir_recursive(
            PathBuf::from(self.from),
            PathBuf::from(self.to),
            PathBuf::new(),
            pool_ref.sender.clone(),
            self._root.as_ref().unwrap().clone(),
            self.verbose
        )
        .expect("Copy failed");

        let now = Instant::now();
        println!("Waiting copy stop...");
        // This loop check if root directory node is copied, which only possible when all children (and
        // children of children, and so on...) of root is copied.
        loop {
            match self._root.as_ref().unwrap().inner().try_read() {
                Ok(reader) => {
                    if reader.is_copied() {
                        println!("Copy complete.");
                        break;
                    }
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        let elapsed_time = now.elapsed();
        println!(
            "Copy action took {} milliseconds.",
            elapsed_time.as_millis()
        );
    }

    pub fn run_single_threads(self) {
        let now = Instant::now();
        Self::copy_dir_recursive_single_thread(&self.from, &self.to, &PathBuf::new(), self.verbose)
            .expect("Copy failed");
        let elapsed_time = now.elapsed();
        println!("Copy action took {} milliseconds", elapsed_time.as_millis());
    }

    pub fn run(self) {
        if self.multi_threads {
            self.run_multi_threads();
        } else {
            self.run_single_threads();
        }
    }

    // TODO May causing problem when handle really big file
    fn copy_file(from: &Path, to: &Path) -> Result<(), io::Error> {
        let content = fs::read_to_string(from)?;
        let mut file = fs::File::create(to)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn copy_dir_recursive_single_thread(
        from: &Path,
        dest: &Path,
        depth_path: &PathBuf,
        verbose: bool
    ) -> Result<(), io::Error> {
        let read_dir = from.clone().to_path_buf().join(depth_path);
        if verbose {
            println!("-----------");
            println!("from : {:?}", from);
            println!("dest : {:?}", dest);
            println!("depth_path : {:?}", depth_path);
            println!("read_dir : {:?}", read_dir);
        }
        for entry in fs::read_dir(read_dir)? {
            let entry = entry?;
            let path = entry.path();
            let new_depth_path = depth_path.clone().join(Path::new(&entry.file_name()));
            let creating_path = dest.clone().to_path_buf().join(new_depth_path.clone());
            if path.is_dir() && !path.is_symlink() {
                if verbose {
                    println!("next depth's path: {:?}", new_depth_path);
                    println!("creating path: {:?}", new_depth_path);
                }
                create_dir_all(&creating_path)?;
                Self::copy_dir_recursive_single_thread(from, dest, &new_depth_path,verbose)?;
            } else if path.is_file() || path.is_symlink() {
                let read_file = from.clone().to_path_buf().join(new_depth_path.clone());
                if verbose {
                    println!("creating file : {:?}", creating_path);
                }
                Self::copy_file(read_file.as_path(), creating_path.as_path())?;
            }
        }

        Ok(())
    }

    fn copy_dir_recursive(
        from: PathBuf,
        dest: PathBuf,
        depth_path: PathBuf,
        sender: Sender<Message>,
        parent_node: SharedNodeRef,
        verbose: bool
    ) -> Result<(), io::Error> {
        let read_dir = from.clone().to_path_buf().join(depth_path.clone());
        if verbose {
            println!("-----------");
            println!("from : {:?}", from);
            println!("dest : {:?}", dest);
            println!("depth_path : {:?}", depth_path);
            println!("read_dir : {:?}", read_dir);
        }
        for entry in fs::read_dir(read_dir)? {
            let entry = entry?;
            let path = entry.path();
            let new_depth_path = depth_path.clone().join(Path::new(&entry.file_name()));
            let creating_path = dest.clone().to_path_buf().join(new_depth_path.clone());
            if path.is_dir() && !path.is_symlink() {
                if verbose {
                    println!("next depth's path: {:?}", new_depth_path);
                    println!("creating path: {:?}", new_depth_path);
                }
                create_dir_all(&creating_path)?;

                // Create new node for directory in this loop, and then attach it to directory tree and
                // set parent for it.
                if verbose {
                    println!("creating new node for path {:?}", creating_path);
                }
                let mut node = DirNode::new(creating_path, verbose);
                node.set_parent(parent_node.clone());
                let node_r = SharedNodeRef::new(node);

                if verbose {
                    println!("add new node to parent");
                }
                let mut writer = parent_node.inner().write().unwrap();
                writer.add_sub_nodes(node_r.clone());
                drop(writer);

                if verbose {
                    println!("attach node to tree done");
                }

                let new_from = from.clone();
                let new_dest = dest.clone();
                let new_new_depth_path = new_depth_path.clone();
                let new_sender = sender.clone();

                //For directory under this directory, make it as a new task to pool.
                sender
                    .send(Message::NewTask(Box::new(move || {
                        Self::copy_dir_recursive(
                            new_from,
                            new_dest,
                            new_new_depth_path,
                            new_sender,
                            node_r,
                            verbose
                        )
                        .unwrap();
                    })))
                    .unwrap();
            } else if path.is_file() || path.is_symlink() {
                let read_file = from.clone().to_path_buf().join(new_depth_path.clone());
                if verbose {
                    println!("creating file : {:?}", creating_path);
                }
                Self::copy_file(read_file.as_path(), creating_path.as_path())?;
            }
        }

        let mut writer = parent_node.inner().write().unwrap();
        if verbose {
            println!("start lookup {:?}", writer.path());
        }
        writer.set_copied(); //当前node的父node检查
        drop(writer);

        DirNode::try_lookup_continuously(parent_node);

        Ok(())
    }
}
#[cfg(test)]
mod copy_test {
    use super::*;

    #[test]
    fn copy_file_test() {
        let from = Path::new("./test_dir/copy_test_dir/origin_file");
        let to = Path::new("./test_dir/copy_test_dir/copied_file1");
        Copyer::copy_file(&from, &to).unwrap();
        let content_from = fs::read_to_string(&from).unwrap();
        let content_to = fs::read_to_string(&to).unwrap();
        assert_eq!(content_to, content_from);
    }
}
