use crate::splay_forest::{SplayForest, NodeIdx};
use crate::aggregate::Aggregate;
use std::fmt::Debug;

pub struct LinkCutTree<V, A: Aggregate<V> = ()> {
    rep: SplayForest<V, A>,
}

impl<V: Debug, A: Aggregate<V>> LinkCutTree<V, A> {
    pub fn new() -> Self {
        LinkCutTree { rep: SplayForest::new() }
    }

    pub fn make_tree(&mut self, val: V) -> NodeIdx {
        self.rep.add_node(val)
    }

    pub fn access(&mut self, node_idx: NodeIdx) {
        self.rep.splay(node_idx);
        self.rep.split_right_and_attach_new(node_idx, None);
        let mut v = node_idx;
        loop {
            match self.rep.get_path_parent(v) {
                None => { break; }
                Some(w) => {
                    self.rep.splay(w);
                    self.rep.split_right_and_attach_new(w, v.into());
                    v = w;
                }
            }
        }
        self.rep.splay(node_idx);
    }

    pub fn find_root(&mut self, node_idx: NodeIdx) -> NodeIdx {
        self.access(node_idx);
        let root = self.rep.get_leftmost(node_idx);
        self.access(root);
        return root;
    }

    pub fn cut(&mut self, node_idx: NodeIdx) {
        self.access(node_idx);
        self.rep.split_left(node_idx);
    }

    pub fn link(&mut self, parent_idx: NodeIdx, child_idx: NodeIdx) {
        self.access(child_idx);
        self.access(parent_idx);
        self.rep.join_left(child_idx, parent_idx);
    }

    pub fn get_val(&mut self, node_idx: NodeIdx) -> &V {
        self.rep.get_value(node_idx)
    }

    pub fn set_val(&mut self, node_idx: NodeIdx, val: V) {
        self.rep.set_value(node_idx, val);
    }

    /// Returns the aggregate of the values on the path from the root of
    /// `node_idx`'s tree down to `node_idx`, inclusive.
    ///
    /// After `access`, the node's auxiliary tree contains exactly the nodes
    /// on the root-to-node path, so the subtree aggregate at its root is the
    /// path aggregate.
    pub fn path_aggregate(&mut self, node_idx: NodeIdx) -> A {
        self.access(node_idx);
        self.rep.get_aggregate(node_idx).clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::link_cut_tree::LinkCutTree;
    use crate::aggregate::{Aggregate, Sum, Max};
    use crate::splay_forest::NodeIdx;

    #[test]
    fn basic_tree() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("1");
        assert_eq!(lct.find_root(node1), node1);
    }

    #[test]
    fn multiple_roots() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("1");
        let node2 = lct.make_tree("2");
        assert_eq!(lct.find_root(node1), node1);
        assert_eq!(lct.find_root(node2), node2);
        assert_eq!(lct.find_root(node1), node1);
    }

    #[test]
    fn link_basic() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("1");
        let node2 = lct.make_tree("2");
        lct.link(node1, node2);
        assert_eq!(lct.find_root(node1), node1, "original root changed");
        assert_eq!(lct.find_root(node2), node1, "new root not updated");
        assert_eq!(lct.find_root(node1), node1, "original root changed after one iteration");
        assert_eq!(lct.find_root(node1), node1, "original root changed after repetition");
        assert_eq!(lct.find_root(node2), node1, "new root unupdated");
    }

    #[test]
    fn link_multiple() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("Grandparent");
        let node2 = lct.make_tree("Parent");
        let node3 = lct.make_tree("Child");
        lct.link(node1, node2);
        lct.link(node2, node3);
        assert_eq!(lct.find_root(node3), node1, "Didn't find grandparent");
        assert_eq!(lct.find_root(node2), node1, "Didn't find parent");
        assert_eq!(lct.find_root(node2), node1, "Didn't find itself");
    }

    #[test]
    fn many_links_to_one() {
        let mut lct: LinkCutTree<String> = LinkCutTree::new();
        let node1 = lct.make_tree("Parent".into());
        let children = (1..6).map(
            |i| lct.make_tree(format!("Child {}", i))
        ).collect::<Vec<_>>();
        for child in &children {
            lct.link(node1, *child);
        }

        for child in &children {
            assert_eq!(lct.find_root(*child), node1, "Wrong parent");
        }
    }

    #[test]
    fn many_links_to_one_root() {
        let mut lct: LinkCutTree<String> = LinkCutTree::new();
        let root = lct.make_tree("Root".into());
        let mut all_nodes = vec![root];
        let mut last_depth_nodes = vec![root];
        let mut current_depth_nodes = vec![];
        for _depth in 0..3 {
            for parent_idx in &last_depth_nodes {
                for child_num in 1..3 {
                    let child_name = format!("{}({})", lct.get_val(*parent_idx), child_num);
                    let child_idx = lct.make_tree(child_name);
                    lct.link(*parent_idx, child_idx);
                    current_depth_nodes.push(child_idx);
                    all_nodes.push(child_idx);
                }
            }
            last_depth_nodes.clear();
            last_depth_nodes.append(&mut current_depth_nodes);
        }

        for node in all_nodes {
            assert_eq!(lct.find_root(node), root, "Failed on node: {}", lct.get_val(node));
        }
    }

    #[test]
    fn test_cut_basic() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("1");
        let node2 = lct.make_tree("2");
        lct.link(node1, node2);
        assert_eq!(lct.find_root(node1), node1, "original root changed");
        assert_eq!(lct.find_root(node2), node1, "new root not updated");
        lct.cut(node2);
        assert_eq!(lct.find_root(node1), node1, "should still be its own root");
        assert_eq!(lct.find_root(node2), node2, "should be back to being its own root");
    }

    #[test]
    fn path_sum_chain() {
        let mut lct: LinkCutTree<i64, Sum<i64>> = LinkCutTree::new();
        let nodes = (0..100).map(|i| lct.make_tree(i)).collect::<Vec<_>>();
        for pair in nodes.windows(2) {
            lct.link(pair[0], pair[1]);
        }
        for (i, node) in nodes.iter().enumerate() {
            let expected: i64 = (0..=i as i64).sum();
            assert_eq!(lct.path_aggregate(*node), Sum(expected), "wrong sum at depth {}", i);
        }
    }

    #[test]
    fn path_sum_branching() {
        // 1 -> 2 -> 4 and 1 -> 3 -> 5
        let mut lct: LinkCutTree<i64, Sum<i64>> = LinkCutTree::new();
        let n1 = lct.make_tree(1);
        let n2 = lct.make_tree(2);
        let n3 = lct.make_tree(3);
        let n4 = lct.make_tree(4);
        let n5 = lct.make_tree(5);
        lct.link(n1, n2);
        lct.link(n1, n3);
        lct.link(n2, n4);
        lct.link(n3, n5);
        assert_eq!(lct.path_aggregate(n4), Sum(1 + 2 + 4));
        assert_eq!(lct.path_aggregate(n5), Sum(1 + 3 + 5));
        assert_eq!(lct.path_aggregate(n1), Sum(1));
    }

    #[test]
    fn path_sum_after_cut_and_relink() {
        let mut lct: LinkCutTree<i64, Sum<i64>> = LinkCutTree::new();
        let n1 = lct.make_tree(1);
        let n2 = lct.make_tree(2);
        let n3 = lct.make_tree(3);
        lct.link(n1, n2);
        lct.link(n2, n3);
        assert_eq!(lct.path_aggregate(n3), Sum(6));

        lct.cut(n2);
        assert_eq!(lct.path_aggregate(n3), Sum(5), "cut subtree should keep only its own path");
        assert_eq!(lct.path_aggregate(n1), Sum(1), "remaining tree should exclude cut subtree");

        lct.link(n3, n1);
        assert_eq!(lct.path_aggregate(n1), Sum(2 + 3 + 1), "relinked path should aggregate through new root");
    }

    #[test]
    fn path_sum_after_set_val() {
        let mut lct: LinkCutTree<i64, Sum<i64>> = LinkCutTree::new();
        let n1 = lct.make_tree(1);
        let n2 = lct.make_tree(2);
        lct.link(n1, n2);
        assert_eq!(lct.path_aggregate(n2), Sum(3));
        lct.set_val(n1, 10);
        assert_eq!(lct.path_aggregate(n2), Sum(12));
    }

    #[test]
    fn path_max() {
        let mut lct: LinkCutTree<i64, Max<i64>> = LinkCutTree::new();
        let n1 = lct.make_tree(7);
        let n2 = lct.make_tree(3);
        let n3 = lct.make_tree(9);
        lct.link(n1, n2);
        lct.link(n2, n3);
        assert_eq!(lct.path_aggregate(n1), Max(7));
        assert_eq!(lct.path_aggregate(n2), Max(7));
        assert_eq!(lct.path_aggregate(n3), Max(9));
    }

    /// A non-commutative aggregation: concatenating path values in
    /// root-to-node order. Verifies that `combine` always sees the shallower
    /// segment as `upper`, regardless of how the splay trees are shaped.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct PathConcat(String);

    impl Aggregate<&str> for PathConcat {
        fn from_value(value: &&str) -> Self {
            PathConcat(value.to_string())
        }

        fn combine(upper: &Self, lower: &Self) -> Self {
            PathConcat(format!("{}{}", upper.0, lower.0))
        }
    }

    #[test]
    fn path_concat_is_ordered_root_to_node() {
        let mut lct: LinkCutTree<&str, PathConcat> = LinkCutTree::new();
        let names = ["a", "b", "c", "d", "e", "f"];
        let nodes = names.iter().map(|name| lct.make_tree(*name)).collect::<Vec<_>>();
        for pair in nodes.windows(2) {
            lct.link(pair[0], pair[1]);
        }
        // Query out of order to force plenty of splay restructuring.
        assert_eq!(lct.path_aggregate(nodes[3]), PathConcat("abcd".into()));
        assert_eq!(lct.path_aggregate(nodes[5]), PathConcat("abcdef".into()));
        assert_eq!(lct.path_aggregate(nodes[1]), PathConcat("ab".into()));
        assert_eq!(lct.path_aggregate(nodes[4]), PathConcat("abcde".into()));
        assert_eq!(lct.path_aggregate(nodes[0]), PathConcat("a".into()));
    }

    /// A trivially correct forest with parent pointers, used as a reference
    /// implementation for the randomized test below.
    struct NaiveForest {
        parent: Vec<Option<usize>>,
        values: Vec<i64>,
    }

    impl NaiveForest {
        fn find_root(&self, mut v: usize) -> usize {
            while let Some(p) = self.parent[v] {
                v = p;
            }
            v
        }

        fn path_sum(&self, v: usize) -> i64 {
            let mut sum = self.values[v];
            let mut cur = v;
            while let Some(p) = self.parent[cur] {
                sum += self.values[p];
                cur = p;
            }
            sum
        }
    }

    /// Deterministic PRNG (xorshift) so the test is reproducible.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self, bound: usize) -> usize {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            (x % bound as u64) as usize
        }
    }

    #[test]
    fn randomized_against_naive() {
        const N: usize = 50;
        const OPS: usize = 2000;

        let mut rng = Rng(0x853c49e6748fea9b);
        let mut lct: LinkCutTree<i64, Sum<i64>> = LinkCutTree::new();
        let mut naive = NaiveForest { parent: vec![None; N], values: Vec::new() };
        let mut nodes = Vec::new();
        for i in 0..N {
            let val = (i as i64) * 31 % 97 - 40;
            nodes.push(lct.make_tree(val));
            naive.values.push(val);
        }

        for op in 0..OPS {
            match rng.next(4) {
                0 => {
                    // Link a root of one tree under a node of a different tree.
                    let child = rng.next(N);
                    let parent = rng.next(N);
                    if naive.parent[child].is_none()
                        && naive.find_root(parent) != child {
                        lct.link(nodes[parent], nodes[child]);
                        naive.parent[child] = Some(parent);
                    }
                }
                1 => {
                    // Cut a non-root node from its parent.
                    let v = rng.next(N);
                    if naive.parent[v].is_some() {
                        lct.cut(nodes[v]);
                        naive.parent[v] = None;
                    }
                }
                2 => {
                    let v = rng.next(N);
                    assert_eq!(
                        lct.find_root(nodes[v]),
                        NodeIdx::new(naive.find_root(v)),
                        "find_root mismatch at op {}", op
                    );
                }
                _ => {
                    let v = rng.next(N);
                    assert_eq!(
                        lct.path_aggregate(nodes[v]),
                        Sum(naive.path_sum(v)),
                        "path sum mismatch for node {} at op {}", v, op
                    );
                }
            }
        }
    }

    #[test]
    fn many_cut_links() {
        let mut lct: LinkCutTree<String> = LinkCutTree::new();
        let node1 = lct.make_tree("Parent".into());
        let children = (1..6).map(
            |i| lct.make_tree(format!("Child {}", i))
        ).collect::<Vec<_>>();
        for child in &children {
            lct.link(node1, *child);
        }

        for child in &children {
            lct.cut(*child)
        }

        for child in &children {
            assert_eq!(lct.find_root(*child), *child, "Wrong parent");
        }
        assert_eq!(lct.find_root(node1), node1);
    }

}