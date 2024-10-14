/// Node types for templating search functions.
///
/// Implementors of this trait should be ZSTs that are used to template search
/// functions, which can query static compile-time constants to control their
/// behavior in different types of nodes.
///
/// See the node types [`Root`], [`PV`], and [`NonPV`];
pub trait NodeType {
    /// Whether this node is on the principal variation.
    const IS_PV: bool;
    /// Whether this node is the root node of the search tree.
    const IS_ROOT: bool;
    /// The node type that follows in a PV search from this node.
    type Next: NodeType;
}

/// The root node of the search tree.
///
/// This node is considered to be on the principal variation.
pub struct Root;

impl NodeType for Root {
    const IS_PV: bool = true;
    const IS_ROOT: bool = true;
    type Next = PV;
}

/// A node on the principal variation, but not the root node.
pub struct PV;

impl NodeType for PV {
    const IS_PV: bool = true;
    const IS_ROOT: bool = false;
    type Next = PV;
}

/// A node off the principal variation.
pub struct NonPV;

impl NodeType for NonPV {
    const IS_PV: bool = false;
    const IS_ROOT: bool = false;
    type Next = NonPV;
}
