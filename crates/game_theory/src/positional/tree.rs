use std::{process::Child, rc::Rc};

#[derive(Debug)]
pub struct ArenaTree<T> {
    arena: Vec<Node<T>>,
    root: usize,
}

impl<T> ArenaTree<T> {
    pub fn new(value: T) -> Self {
        let arena = vec![Node {
            id: 0,
            value,
            parent: None,
        }];
        Self { arena, root: 0 }
    }

    pub fn root_mut(&mut self) -> &mut Node<T> {
        self.arena.first_mut().unwrap()
    }

    pub fn root(&self) -> &Node<T> {
        self.arena.first().unwrap()
    }

    pub fn add_child(&mut self, parent: usize, value: T) -> NodeMutView<'_, T> {
        let id = self.arena.len();
        assert!(parent < id);

        self.arena.push(Node {
            id,
            parent: Some(parent),
            value,
        });

        NodeMutView { tree: self, id }
    }
}

#[derive(Debug)]
pub struct Node<T> {
    id: usize,
    value: T,
    parent: Option<usize>,
}

pub struct NodeMutView<'a, T> {
    pub id: usize,
    tree: &'a mut ArenaTree<T>,
}

impl<T> Node<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_tree() {
        let mut tree = ArenaTree::new("root");
        tree.add_child(0, "foo");
        tree.add_child(0, "foo");

        println!("Tree: {tree:?}");
    }
}
