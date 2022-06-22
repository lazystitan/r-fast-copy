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

    fn copied(&mut self) {
        self._is_copied = true;
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
        self._sub_nodes.retain(|_k, r| {
            //delete those nodes that had been copied
            let mut writer = r.0.write().unwrap();
            !writer.check_all_copied()
        });
        return self._sub_nodes.is_empty() && self.is_copied();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::{RefCell, RefMut};
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn tree_test() {
        let r = DirNode::new(PathBuf::from(String::from("./test_dir/tree")));
        let root_rc = SharedNodeRef::new(r);

        for i in 0..5 {
            let p = PathBuf::from("./test_dir/tree/c".to_string() + &i.to_string());
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
        let r2 = sub_r._sub_nodes.get("./test_dir/tree/c2").unwrap();

        for i in 0..5 {
            let p = PathBuf::from("./test_dir/tree/c2/d".to_string() + &i.to_string());
            let mut tn = DirNode::new(p);
            tn.set_parent(r2.clone());
            r2.0.write().unwrap().add_sub_nodes(SharedNodeRef::new(tn));
        }
        drop(sub_r);

        println!("start");

        for (_key, r) in &root_rc.0.read().unwrap()._sub_nodes {
            println!("sub node {:?}", r.0.read().unwrap().path());
            if !r.0.read().unwrap()._sub_nodes.is_empty() {
                println!("-------------");
                for (_key2, r2) in &r.0.read().unwrap()._sub_nodes {
                    println!("sub node {:?}", r2.0.read().unwrap().path());
                    r2.0.write().unwrap().copied();
                }
                println!("-------------");
                let mut r_bm = r.0.write().unwrap();
                r_bm.copied();
            }
        }

        // let new_r = new_r.clone();
        new_r.0.write().unwrap().check_all_copied();

        println!("------change-------");

        for (_key, r) in &root_rc.0.read().unwrap()._sub_nodes {
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
