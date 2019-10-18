
#![allow(unused)]
use serde::{Serialize, Deserialize};
use std::rc::Rc;
use std::cell::RefCell;

pub struct Node<T> {
    pub data: T,
    pub edges: Option<Vec<Link<T>>>,
}

pub struct Link<T> {
    pub weight: usize,
    pub from: Rc<RefCell<Node<T>>>,
    pub to: Rc<RefCell<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(s: T) -> Self {
        Node{
            data: s,
            edges: None,
        }
    }
    
    pub fn is_leaf(&self) -> bool {
        self.edges.is_none()
    }
    
    pub fn walk<F>(&self, f: &mut F) where F: FnMut(&Node<T>) {
        match &self.edges {
            Some(children) => {
                for child in children {
                    child.to.borrow().walk(f);
                }
            },
            _ => (),
        }
        
        f(self);
    }
}

pub struct Graph<T> {
    pub root: Option<Rc<RefCell<Node<T>>>>,
    pub area: Vec<Rc<RefCell<Node<T>>>>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Graph{
            root: None,
            area: Vec::new(),
        }
    }

    pub fn add_node(&mut self, d: T) -> Rc<RefCell<Node<T>>> {
        let s = Rc::new(RefCell::new(Node::new(d)));
        self.area.push(s.clone());

        if self.root.is_none() {
            self.root = Some(s.clone());
        }

        s
    }

    pub fn link(l: Rc<RefCell<Node<T>>>,r: Rc<RefCell<Node<T>>>, w: usize) {
        let link = Link{from: l.clone(), to: r.clone(), weight: w};
        let mut ls = l.borrow_mut();
        match &mut ls.edges {
            Some(edges) => edges.push(link),
            None => ls.edges = Some(vec![link]),
        }
    }

    pub fn count(&self) -> i64 {
        let(count, _) = Graph::go(self.root.clone().unwrap());
        count
    }

    pub fn deep(&self) -> i64 {
        let(_, d) = Graph::go(self.root.clone().unwrap());
        d
    }

    pub fn go(n: Rc<RefCell<Node<T>>>) -> (i64, i64) {
        let (mut count, mut deep) = (1, 1);
        if let Some(children) = &n.borrow().edges {
            let mut deep_leafs = Vec::with_capacity(children.len());
            for child in children {
                let (lc, ld) = Graph::go(child.to.clone());
                count += lc;
                deep_leafs.push(ld);
            }
            deep += deep_leafs.iter().max().unwrap();
        }
        
        (count, deep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leaf() {
        let mut root = Node::new("maxim");
        
        assert!(root.is_leaf());
        
        root.edges = Some(Vec::new());
        assert_eq!(root.is_leaf(), false);
    }

    #[test]
    fn test_deep() {
        let tree = example();
        
        assert_eq!(tree.deep(), 4);
    }

    #[test]
    fn test_deep_and_count_in_root_tree() {
        let mut tree = Graph::new();
        tree.add_node("example");
        
        assert_eq!(tree.deep(), 1);
        assert_eq!(tree.count(), 1);
    }

    #[test]
    fn test_count() {
        let tree = example();
        
        assert_eq!(tree.count(), 6);
    }
    
    fn example() -> Graph<&'static str> {
        let mut graph = Graph::new();
        let a = graph.add_node("d: T");
        let b = graph.add_node("world");
        let c = graph.add_node("1234");
        let d = graph.add_node("@@@@@");
        let w = graph.add_node("t");
        let q = graph.add_node("a");
        
        Graph::link(a.clone(), b.clone(), 10);
        Graph::link(a.clone(), c.clone(), 20);
        Graph::link(c.clone(), d.clone(), 30);
        Graph::link(d.clone(), w.clone(), 1);
        Graph::link(d.clone(), q.clone(), 12);
       
   
        graph
    }
}
