use std::ops::Add;

/// A path aggregation, as described in section 3.3 of the
/// [6.851 lecture notes](https://courses.csail.mit.edu/6.851/spring12/scribe/L19.pdf).
///
/// Each auxiliary (splay) tree node stores the aggregate of its splay subtree,
/// which corresponds to a contiguous slice of one preferred path in the
/// represented tree. The invariant maintained by the splay forest is:
///
/// ```text
/// node.agg = combine(combine(left.agg, from_value(node.value)), right.agg)
/// ```
///
/// where the left subtree holds the shallower (closer to the root) part of the
/// path and the right subtree holds the deeper part. `combine` is therefore
/// always called with the shallower segment as `upper`, so non-commutative
/// aggregations observe values in root-to-node order. `combine` must be
/// associative.
///
/// To aggregate *edge* weights instead of node values, store each edge's
/// weight in its child node and use an identity value at the root.
pub trait Aggregate<V>: Clone {
    /// The aggregate of a single-node path.
    fn from_value(value: &V) -> Self;

    /// Combines the aggregates of two adjacent path segments, where `upper`
    /// is the segment closer to the root of the represented tree.
    fn combine(upper: &Self, lower: &Self) -> Self;
}

/// The trivial aggregation, used by default when no aggregation is needed.
impl<V> Aggregate<V> for () {
    fn from_value(_value: &V) -> Self {}
    fn combine(_upper: &Self, _lower: &Self) -> Self {}
}

/// Aggregates the sum of the values on the path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sum<T>(pub T);

impl<T: Add<Output = T> + Copy> Aggregate<T> for Sum<T> {
    fn from_value(value: &T) -> Self {
        Sum(*value)
    }

    fn combine(upper: &Self, lower: &Self) -> Self {
        Sum(upper.0 + lower.0)
    }
}

/// Aggregates the maximum value on the path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Max<T>(pub T);

impl<T: Ord + Copy> Aggregate<T> for Max<T> {
    fn from_value(value: &T) -> Self {
        Max(*value)
    }

    fn combine(upper: &Self, lower: &Self) -> Self {
        Max(upper.0.max(lower.0))
    }
}

/// Aggregates the minimum value on the path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Min<T>(pub T);

impl<T: Ord + Copy> Aggregate<T> for Min<T> {
    fn from_value(value: &T) -> Self {
        Min(*value)
    }

    fn combine(upper: &Self, lower: &Self) -> Self {
        Min(upper.0.min(lower.0))
    }
}
