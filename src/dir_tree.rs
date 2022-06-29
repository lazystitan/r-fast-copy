use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// Wrapper of reference of node for convenience.
pub struct SharedNodeRef(Arc<RwLock<DirNode>>);

impl SharedNodeRef {
    pub fn new(n: DirNode) -> Self {
        Self(Arc::new(RwLock::new(n)))
    }

    pub fn inner(&self) -> &Arc<RwLock<DirNode>> {
        &self.0
    }
}

impl Clone for SharedNodeRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl PartialEq for SharedNodeRef {
    fn eq(&self, other: &Self) -> bool {
        let self_arc = self.0.clone();
        let self_reader = self_arc.read().unwrap();
        let self_path = self_reader.path();

        let other_arc = other.0.clone();
        let other_reader = other_arc.read().unwrap();
        let other_path = other_reader.path();
        self_path == other_path
    }
}

impl Eq for SharedNodeRef {}

impl Hash for SharedNodeRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.read().unwrap().path().hash(state);
    }
}

pub struct DirNode {
    _parent: Option<SharedNodeRef>,
    _path: PathBuf,
    _is_copied: bool,
    _sub_nodes: HashMap<String, SharedNodeRef>,
    verbose: bool
}

impl DirNode {
    pub fn new(path: PathBuf, verbose: bool) -> Self {
        Self {
            _parent: None,
            _path: path,
            _is_copied: false,
            _sub_nodes: HashMap::new(),
            verbose,
        }
    }

    pub fn parent(&self) -> &Option<SharedNodeRef> {
        &self._parent
    }

    pub fn add_sub_nodes(&mut self, node: SharedNodeRef) {
        let key = node.0.read().unwrap().path().to_str().unwrap().to_string();
        self._sub_nodes.insert(key, node);
    }

    pub fn set_copied(&mut self) {
        //If leaf, set copied.
        if self._sub_nodes.is_empty() {
            if self.verbose {
                println!(
                    "{:?} has no children, is leaf. Set copied true.",
                    self._path
                );
            }
            self._is_copied = true;
            return;
        }
        //Try delete children when is not leaf.
        if self.verbose {
            println!(
                "{:?} has children, is not leaf, try delete children.",
                self._path
            );
        }
        self._sub_nodes.retain(|_k, r| {
            //delete those nodes that had been copied
            let read = r.0.read().unwrap();
            if self.verbose {
                println!("{:?} has read lock when judge if to delete", read._path);
            }
            let r = !read.is_copied();
            if self.verbose {
                println!("{:?}'s read lock drop when judge if to delete", read._path);
            }
            drop(read);
            r
        });

        //If children is empty then is leaf, set copied.
        self._is_copied = self._sub_nodes.is_empty();

        if self._is_copied && self.verbose {
            println!(
                "{:?} has no children after delete, is leaf. return.",
                self._path
            );
        } else if self.verbose {
            println!(
                "{:?} has children after delete, is not leaf. return.",
                self._path
            );
        }
    }

    pub fn is_copied(&self) -> bool {
        return self._is_copied;
    }

    pub fn set_parent(&mut self, p: SharedNodeRef) {
        self._parent = Some(p);
    }

    pub fn path(&self) -> &PathBuf {
        &self._path
    }

    // If current node is copied and has parent, set current node to parent and repeat.
    pub fn try_lookup_continuously(start_node: SharedNodeRef) {
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
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::{RefCell, RefMut};
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::TryLockError;
    use std::thread;
    use std::time::Duration;

    //build test tree
    //  ----------------------------------<tree> --------------------------------------
    //  |       |                            |                               |         |
    // <A1>    <A2>      ------------------ <A3> --------------------       <A4>      <A5>
    //                   |          |         |         |           |
    //                  <B1>      <B2>      <B3>       <B4>        <B5>
    fn build_tree() -> SharedNodeRef {
        let r = DirNode::new(PathBuf::from(String::from("./test_dir/tree")), true);
        let root_rc = SharedNodeRef::new(r);

        for i in 0..5 {
            let p = PathBuf::from("./test_dir/tree/A".to_string() + &(i + 1).to_string());
            let mut tn = DirNode::new(p, true);
            tn.set_parent(root_rc.clone());
            root_rc
                .0
                .write()
                .unwrap()
                .add_sub_nodes(SharedNodeRef::new(tn));
        }

        let new_r = root_rc.clone();
        let sub_r = new_r.0.read().unwrap();
        let r2 = sub_r._sub_nodes.get("./test_dir/tree/A3").unwrap();

        for i in 0..5 {
            let p = PathBuf::from("./test_dir/tree/A3/B".to_string() + &(i + 1).to_string());
            let mut tn = DirNode::new(p, true);
            tn.set_parent(r2.clone());
            r2.0.write().unwrap().add_sub_nodes(SharedNodeRef::new(tn));
        }
        drop(sub_r);
        root_rc
    }

    //Single thread test
    #[test]
    fn tree_test_threads() {
        let root = build_tree();
        let mut handlers = vec![];

        println!("start");

        for (_key, r) in &root.0.read().unwrap()._sub_nodes {
            let shared_node = r.0.clone();
            handlers.push(thread::spawn(move || {
                let mut writer = shared_node.write().unwrap();
                writer.set_copied();
                drop(writer);
                let reader = shared_node.read().unwrap();
                let lookup_flag = reader.is_copied();
                if lookup_flag {
                    let mut p = None;
                    if let Some(inner_p) = &reader._parent {
                        p = Some(inner_p.clone());
                    }
                    println!("lookup {:?}", reader.path());
                    drop(reader);
                    if let Some(p) = p {
                        let mut writer = p.0.write().unwrap();
                        writer.set_copied();
                    }
                }
            }));

            let reader = r.0.read().unwrap();
            if !reader._sub_nodes.is_empty() {
                for (_, r) in &reader._sub_nodes {
                    let shared_node = r.0.clone();
                    handlers.push(thread::spawn(move || {
                        let mut writer = shared_node.write().unwrap();
                        writer.set_copied();
                        drop(writer);
                        let reader = shared_node.read().unwrap();
                        let lookup_flag = reader.is_copied();
                        if lookup_flag {
                            let mut p = None;
                            if let Some(inner_p) = &reader._parent {
                                p = Some(inner_p.clone());
                            }
                            println!("lookup {:?}", reader.path());
                            drop(reader);
                            if let Some(p) = p {
                                let mut writer = p.0.write().unwrap();
                                writer.set_copied();
                            }
                        }
                    }))
                }
            }
        }

        println!("Waiting copy stop...");
        loop {
            println!("in loop");
            match root.0.try_read() {
                Ok(reader) => {
                    if reader.is_copied() {
                        println!("Copy complete.");
                        break;
                    }
                }
                Err(e) => match e {
                    TryLockError::Poisoned(_) => {}
                    TryLockError::WouldBlock => {}
                },
            }
            thread::sleep(Duration::from_millis(50));
        }

        for h in handlers {
            h.join().unwrap();
        }

        println!("done.");
    }

    #[test]
    fn ref_cell_test() {
        let shared_map: Rc<RefCell<_>> = Rc::new(RefCell::new(HashMap::new()));
        let mut map: RefMut<_> = (*shared_map).borrow_mut();
        map.insert("africa", 92388);
        map.insert("kyoto", 11837);
        map.insert("piccadilly", 11826);
        map.insert("marbles", 38);
        println!("{:?}", shared_map);

        let shared_map: Rc<RefCell<_>> = Rc::new(RefCell::new(HashMap::new()));
        shared_map.borrow_mut().insert("africa", 92388);
        shared_map.borrow_mut().insert("kyoto", 11837);
        shared_map.borrow_mut().insert("piccadilly", 11826);
        shared_map.borrow_mut().insert("marbles", 38);
        println!("{:?}", shared_map);
    }
}
