//! Core types of the tree abstraction.

use crate::{forest::util::ImSlice, FieldKey, TreeType};





/// Generic indexing trait.
/// Based on https://www.reddit.com/r/rust/comments/qce86d/generalizing_with_gat_whats_going_to_happen_to/
pub trait Indexable {
    type Item;

    fn index(&self, index: usize) -> Self::Item;
    fn len(&self) -> usize;
}

impl<'a, T> Indexable for &'a [T] {
    type Item = &'a T;

    fn index(&self, i: usize) -> Self::Item {
        self.get(i).unwrap()
    }
    fn len(&self) -> usize {
        (self as &'_ [T]).len()
    }
}

/// Navigation part of Node
pub trait NodeNav<'a> {
    /// For indexing children within a field.
    type TField: Indexable<Item = Self>;

    /// For iterating the set of field labels for non-empty fields.
    type TFields: Iterator<Item = (&'a FieldKey, Self::TField)>;

    fn get_field(&self, label: FieldKey) -> Self::TField;
    fn get_fields(&self) -> Self::TFields;
}

/// Tree Node.
/// Combines navigation with data (def and payload)
pub trait NodeData {
    fn get_def(&self) -> TreeType;
    fn get_payload(&self) -> Option<ImSlice>;
}

pub trait Node<'a>: NodeNav<'a> + NodeData {}

impl<'a, TNode: NodeData + NodeNav<'a>> Node<'a> for TNode {}

/// Information about the parent of a Node.
#[derive(Clone)]
pub struct ParentInfo<TNode> {
    pub node: TNode,
    pub label: FieldKey,
}
