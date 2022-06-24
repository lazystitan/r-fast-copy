use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// pub type SharedNodeRef = Rc<RefCell<DirNode>>;

pub struct SharedNodeRef(Arc<RwLock<DirNode>>);

impl SharedNodeRef {
    fn new(n: DirNode) -> Self {
        Self(Arc::new(RwLock::new(n)))
    }
}

impl Clone for SharedNodeRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// impl Deref for SharedNodeRef {
//     type Target = Rc<RefCell<DirNode>>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

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
}

impl DirNode {
    fn new(path: PathBuf) -> Self {
        Self {
            _parent: None,
            _path: path,
            _is_copied: false,
            _sub_nodes: HashMap::new(),
        }
    }

    fn add_sub_nodes(&mut self, node: SharedNodeRef) {
        let key = node.0.read().unwrap().path().to_str().unwrap().to_string();
        self._sub_nodes.insert(key, node);
    }

    fn this_node_copied(&mut self) {
        //if leaf
        if self._sub_nodes.is_empty() {
            println!("{:?} has no children, is leaf. Set copied true.", self._path);
            self._is_copied = true;
            return;
        }
        //try delete children
        println!("{:?} has children, is not leaf, try delete children.", self._path);
        self._sub_nodes.retain(|_k, r| {
            //delete those nodes that had been copied
            let read = r.0.read().unwrap();
            println!("{:?} has read lock", read._path);
            let r = !read.is_copied();
            println!("{:?}'s read lock drop", read._path);
            drop(read);
            r
        });
        self._is_copied = self._sub_nodes.is_empty();

        println!("{:?} has children after delete, is not leaf. return.", self._path);
    }

    fn is_copied(&self) -> bool {
        return self._is_copied;
    }

    fn set_parent(&mut self, p: SharedNodeRef) {
        self._parent = Some(p);
    }

    fn path(&self) -> &PathBuf {
        &self._path
    }

    fn check_all_copied(&mut self) -> bool {
        return self.is_copied();
    }

    fn lookup(&self) {
        if let Some(p) = &self._parent {
            let mut writer = p.0.write().unwrap();
            writer.this_node_copied();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::{RefCell, RefMut};
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::{RwLockReadGuard, TryLockError, TryLockResult};
    use std::thread;
    use std::time::Duration;

    //build test tree
    //  ----------------------------------<tree> --------------------------------------
    //  |       |                            |                               |         |
    // <A1>    <A2>      ------------------ <A3> --------------------       <A4>      <A5>
    //                   |          |         |         |           |
    //                  <B1>      <B2>      <B3>       <B4>        <B5>
    fn build_tree() -> SharedNodeRef {
        let r = DirNode::new(PathBuf::from(String::from("./test_dir/tree")));
        let root_rc = SharedNodeRef::new(r);

        for i in 0..5 {
            let p = PathBuf::from("./test_dir/tree/A".to_string() + &(i + 1).to_string());
            let mut tn = DirNode::new(p);
            tn.set_parent(root_rc.clone());
            root_rc
                .0
                .write()
                .unwrap()
                .add_sub_nodes(SharedNodeRef::new(tn));
        }

        let new_r = root_rc.clone();
        let sub_r = new_r.0.read().unwrap();
        let r2 = sub_r._sub_nodes.get("./test_dir/tree/A2").unwrap();

        for i in 0..5 {
            let p = PathBuf::from("./test_dir/tree/c2/B".to_string() + &(i + 1).to_string());
            let mut tn = DirNode::new(p);
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
                writer.this_node_copied();
                drop(writer);
                let reader = shared_node.read().unwrap();
                let mut p = None;
                if let Some(inner_p) = &reader._parent {
                    p = Some(inner_p.clone());
                }
                println!("lookup {:?}", reader.path());
                drop(reader);
                if let Some(p) = p {
                    let mut writer = p.0.write().unwrap();
                    writer.this_node_copied();
                }
            }));

            let reader = r.0.read().unwrap();
            if !reader._sub_nodes.is_empty() {
                for (_, r) in &reader._sub_nodes {
                    let shared_node = r.0.clone();
                    handlers.push(thread::spawn(move || {
                        let mut writer = shared_node.write().unwrap();
                        writer.this_node_copied();
                        drop(writer);
                        let reader = shared_node.read().unwrap();
                        let mut p = None;
                        if let Some(inner_p) = &reader._parent {
                            p = Some(inner_p.clone());
                        }
                        println!("lookup {:?}", reader.path());
                        drop(reader);
                        if let Some(p) = p {
                            let mut writer = p.0.write().unwrap();
                            writer.this_node_copied();
                        }
                    }))
                }
            }
        }

        // let new_r = new_r.clone();
        // root.0.write().unwrap().check_all_copied();

        println!("Waiting copy stop...");
        loop {
            println!("in loop");
            match root.0.try_read() {
                Ok(_) => {
                    println!("Copy complete.");
                    break;
                }
                Err(e) => {
                    match e {
                        TryLockError::Poisoned(_) => {}
                        TryLockError::WouldBlock => {}
                    }
                }
            }
            thread::sleep(Duration::from_millis(50));
        }

        for h in handlers {
            h.join().unwrap();
        }

        println!("done.");


    }


    #[test]
    fn tree_test_multi() {
        let root = build_tree();

        println!("tree built, start test");

        for (_key, r) in &root.0.read().unwrap()._sub_nodes {
            let reader = r.0.read().unwrap();
            println!("sub node {:?}", reader.path());
            if !reader._sub_nodes.is_empty() {
                drop(reader);

                let mut writer = r.0.write().unwrap();
                writer.this_node_copied();
                drop(writer);

                println!("-------------");

                for (_key2, r2) in &r.0.read().unwrap()._sub_nodes {
                    println!("sub node {:?}", r2.0.read().unwrap().path());
                    r2.0.write().unwrap().this_node_copied();
                }
                println!("-------------");
            }
        }

        // let new_r = new_r.clone();
        root.0.write().unwrap().check_all_copied();

        println!("------change-------");

        for (_key, r) in &root.0.read().unwrap()._sub_nodes {
            println!("sub node {:?}", r.0.read().unwrap().path());
            if !r.0.read().unwrap()._sub_nodes.is_empty() {
                println!("-------------");
                for (_key2, r2) in &r.0.read().unwrap()._sub_nodes {
                    println!("sub node {:?}", r2.0.read().unwrap().path());
                }
                println!("-------------");
            }
        }

        println!("done");
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
