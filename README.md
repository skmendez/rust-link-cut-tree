# Link-Cut Tree implementation in Rust

This repository contains a Rust implementation of the amortized logarithmic link-cut tree data structure as described in [this lecture](https://courses.csail.mit.edu/6.851/spring21/lectures/L19).
It supports `link`, `cut`, `find_root`, and `path_aggregate` operations; see [`src/link_cut_tree.rs`](src/link_cut_tree.rs) for the API.

## Path aggregation

`path_aggregate(v)` returns an aggregate (sum, max, min, or anything associative) of the values on the path from the root of `v`'s tree down to `v`, in `O(lg n)` amortized time. It follows section 3.3 of the [lecture's scribe notes](https://courses.csail.mit.edu/6.851/spring12/scribe/L19.pdf): each auxiliary (splay) tree node stores the aggregate of its splay subtree — a contiguous segment of one preferred path — and the aggregate is kept consistent locally through every rotation, split, and join. `path_aggregate(v)` is then just `access(v)` followed by reading the aggregate at `v`, since after an access `v`'s auxiliary tree contains exactly the root-to-`v` path.

The aggregation is chosen via the [`Aggregate`](src/aggregate.rs) trait, the second type parameter of `LinkCutTree` (defaulting to `()`, i.e. no aggregation):

```rust
use link_cut_tree::link_cut_tree::LinkCutTree;
use link_cut_tree::aggregate::Sum;

let mut lct: LinkCutTree<i64, Sum<i64>> = LinkCutTree::new();
let a = lct.make_tree(1);
let b = lct.make_tree(2);
lct.link(a, b);
assert_eq!(lct.path_aggregate(b), Sum(3));
```

`Sum`, `Min`, and `Max` are provided; custom aggregations implement `Aggregate<V>`. `combine` is always called with the shallower path segment first, so non-commutative aggregations (e.g. concatenation) see values in root-to-node order. To aggregate edge weights instead of node values, store each edge's weight in its child node with an identity value at the root.
