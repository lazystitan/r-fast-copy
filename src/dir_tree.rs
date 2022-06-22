use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

pub type SharedNodeRef = Rc<RefCell<DirNode>>;

#[derive(Debug)] //TODO overflowed
pub struct DirNode {
    _parent: Option<SharedNodeRef>,
    _path: PathBuf,
    _is_copied: bool,
    _sub_nodes: Vec<SharedNodeRef>
}

impl DirNode {
    fn new(path: PathBuf) -> Self {
        Self {
            _parent: None,
            _path: path,
            _is_copied: false,
            _sub_nodes: Vec::new()
        }
    }

    fn add_sub_nodes(&mut self, node: SharedNodeRef) {
        self._sub_nodes.push(node);
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

    fn sub_nodes(&self) -> Vec<SharedNodeRef> {
        self._sub_nodes.iter().map(|x| x.clone()).collect()
    }

    fn path(&self) -> &PathBuf {
        &self._path
    }
}


#[cfg(test)]
mod test {
    // use std::borrow::BorrowMut;
    use std::cell::RefMut;
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn tree_test() {
        let r = DirNode::new(PathBuf::from(String::from("./test_dir/tree")));
        let root_rc = Rc::new(RefCell::new(r));
        for i in 0..5 {
            let p = PathBuf::from("./test_dir/tree/c".to_string() + &i.to_string());
            let mut tn = DirNode::new(p);
            tn.set_parent(root_rc.clone());
            root_rc.borrow_mut().add_sub_nodes(Rc::new(RefCell::new(tn)));
        }
        for i in &root_rc.borrow().sub_nodes() {
            println!("sub node {:?}", i.borrow().path());
        }
        // println!("{:?}", root_rc);
    }

    #[test]
    fn ref_cell_test() {
        let mut shared_map: Rc<RefCell<_>> = Rc::new(RefCell::new(HashMap::new()));
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
