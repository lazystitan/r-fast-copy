use std::cell::{RefCell};
use std::rc::Rc;

#[derive(Debug)]
struct Node(i32);

struct HighNode(Node);

fn main() {
    /*
    immutable、mutable和ownership的属性基于variable
     */
    let x = Node(1);
    let mut c = RefCell::new(x);
    let y = &mut c.get_mut().0;
    *y = *y + 1;
    println!("{}", c.get_mut().0);

    let x = Node(2);
    let mut h = HighNode(x);
    let y = &mut h.0.0;
    *y = *y + 1;
    println!("{}", h.0.0);


    let x = Node(8);
    let rc = Rc::new(RefCell::new(x));
    let rc2 = rc.clone();

    println!("{}", (*rc).borrow().0);
    (*rc2).borrow_mut().0 += 1;
    println!("{}", (*rc).borrow().0);
}
