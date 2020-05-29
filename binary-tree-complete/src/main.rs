// NOTE(elsuizo:2020-05-14): este es un ejemplo del libro: Programming Rust
use std::iter::Iterator;

enum BinaryTree<T> {
    Empty,
    NonEmpty(Box<TreeNode<T>>),
}

struct TreeNode<T> {
    element: T,
    left: BinaryTree<T>,
    right: BinaryTree<T>,
}
// NOTE(elsuizo:2020-05-14): esta es una struct de apoyo para el iterator

// the state of an in-order traversal of a `BinaryTree`
struct TreeIter<'a, T: 'a> {
    // A stack of references to tree nodes. Since we use `Vec`s
    // `push` and `pop` methods, the top of the stack is the end of
    // the vector
    // the node the iterator will visit next is at the top of the stack
    // with those ancestors still unvisited below it. if the stack is empty
    // the iteration is over
    unvisited: Vec<&'a TreeNode<T>>,
}

impl<'a, T: 'a> TreeIter<'a, T> {
    fn push_left_edge(&mut self, mut tree: &'a BinaryTree<T>) {
        while let BinaryTree::NonEmpty(ref node) = *tree {
            self.unvisited.push(node);
            tree = &node.left;
        }
    }
}

// NOTE(elsuizo:2020-05-14): podemos darle a BinaryTree un metodo iter que retorne un iterador
// sobre el tree
impl<T> BinaryTree<T> {
    fn iter(&self) -> TreeIter<T> {
        let mut iter = TreeIter {
            unvisited: Vec::new(),
        };
        // set the initial stack
        iter.push_left_edge(self);
        iter
    }
}

impl<'a, T: 'a> IntoIterator for &'a BinaryTree<T> {
    type Item = &'a T;
    type IntoIter = TreeIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        // find the node this iteration must produce or finish the iteration
        let node = match self.unvisited.pop() {
            None => return None,
            Some(n) => n,
        };
        self.push_left_edge(&node.right);
        // produce a reference to this node's value
        Some(&node.element)
    }
}

fn make_node<T>(left: BinaryTree<T>, element: T, right: BinaryTree<T>) -> BinaryTree<T> {
    BinaryTree::NonEmpty(Box::new(TreeNode {
        left,
        element,
        right,
    }))
}

fn main() {
    // build a small tree
    let subtree_l = make_node(BinaryTree::Empty, "mecha", BinaryTree::Empty);
    let subtree_rl = make_node(BinaryTree::Empty, "droid", BinaryTree::Empty);
    let subtree_r = make_node(subtree_rl, "robot", BinaryTree::Empty);
    let tree = make_node(subtree_l, "Jaeger", subtree_r);

    // iterate over it!!!
    let mut v = Vec::new();
    for kind in &tree {
        v.push(*kind);
    }

    assert_eq!(v, ["mecha", "Jaeger", "droid", "robot"]);
}
