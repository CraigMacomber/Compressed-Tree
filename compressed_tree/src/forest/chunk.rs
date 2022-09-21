//! A `Chunk` of a Tree.

use super::{
    tree::{Node},
};

/// Index under Chunk which a node is stored.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Hash)]
pub struct NodeChunkIndex(pub usize);


pub trait HasId {
    fn get_index_in_chunk(&self) -> NodeChunkIndex;
}


/// A `Chunk` of a Tree.
/// Contains 0 or more nodes, all of which must have `NodeId` between (inclusive) some `first_id` and some `max_id`.
/// No chunk within the same forest can have a range of ids that overlaps with any other.
///
/// NodeNav<ChunkId> is used to record chunk level parentage for parent lookup.
pub trait Chunk: Clone + PartialEq {
    /// The representation of Nodes in this Chunk.
    type View: Node + HasId;
    type Expander: Iterator<Item = Self::View>;

    /// gets an node with an index owned by this chunk
    fn get(&self, index: NodeChunkIndex) -> Option<Self::View>;

    fn total_nodes(&self) -> usize;

    fn top_level_nodes(&self) -> Self::Expander;
}
