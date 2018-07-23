//-------------------------------------------------------------------------
//                        binary tree data-structure
//-------------------------------------------------------------------------

// an ordered collection of T
#[derive(Debug)]
enum BinaryTree<T> {
    Empty,
    NonEmpty(Box<TreeNode<T>>)
}

// a part of BinaryTree
#[derive(Debug)]
struct TreeNode<T> {
    element: T,
    left: BinaryTree<T>,
    right: BinaryTree<T>
}


// implementa un metodo para agregar nodos al arbol binario
impl <T: Ord> BinaryTree<T> {
    fn add(&mut self, value: T) {
        match *self {
            BinaryTree::Empty => *self = BinaryTree::NonEmpty(Box::new(TreeNode {
                element: value,
                left: BinaryTree::Empty,
                right: BinaryTree::Empty
            }))
            BinaryTree::NonEmpty(ref mut node) => if value <= node.element {
                node.left.add(value);
            } else {
                node.right.add(value);
            }
        }
    }
}

use self::BinaryTree::*;

fn main() {
    // build a particular node is straightforward
    let jupiter_tree = NonEmpty(Box::new(TreeNode{
        element: "Jupiter",
        left: Empty,
        right: Empty
    }));

    let mercury_tree = NonEmpty(Box::new(TreeNode{
        element: "mercury_tree",
        left: Empty,
        right: Empty
    }));

    let mars_tree = NonEmpty(Box::new(TreeNode{
        element: "Mars",
        left: jupiter_tree,
        right: mercury_tree
    }));

    println!("{:?}", mars_tree);
}
