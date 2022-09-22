//! Core types of the tree abstraction.

use std::slice;

use crate::{forest::util::ImSlice, FieldKey, TreeType};

pub type Def = TreeType;

pub type Label = FieldKey;

/// Generic indexing trait.
/// Based on https://www.reddit.com/r/rust/comments/qce86d/generalizing_with_gat_whats_going_to_happen_to/
pub trait Indexable {
    type Item<'a>: ?Sized
    where
        Self: 'a;

    fn index<'a>(&'a self, index: usize) -> Self::Item<'a>;
    fn len(&self) -> usize;
}

impl<T> Indexable for Vec<T> {
    type Item<'a> = &'a T where Self: 'a;

    fn index<'a>(&'a self, i: usize) -> Self::Item<'a> {
        std::ops::Index::<usize>::index(self, i)
    }
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<T> Indexable for &'_ [T] {
    type Item<'a> = &'a T where Self: 'a;

    fn index<'a>(&'a self, i: usize) -> Self::Item<'a> {
        self.get(i).unwrap()
    }
    fn len(&self) -> usize {
        (self as &'_ [T]).len()
    }
}

/// Navigation part of Node
pub trait FieldMap {
    /// For indexing children within a field.
    /// TODO: constrain to `Indexable<Item<'a> = Self>` and fix lifetime issue with that.
    type TField<'a>: Indexable
    where
        Self: 'a;

    fn get_field(&self, label: Label) -> Self::TField<'_>;
}

/// Navigation part of Node
pub trait NodeNav: FieldMap {
    /// For iterating the set of field labels for non-empty fields.
    type TFields<'a>: Iterator<Item = (&'a Label, <Self as FieldMap>::TField<'a>)>
    where
        Self: 'a;

    fn get_fields(&self) -> Self::TFields<'_>;
}

/// Tree Node.
/// Combines navigation with data (def and payload)
pub trait NodeData {
    fn get_def(&self) -> Def;
    fn get_payload(&self) -> Option<ImSlice>;
}

pub trait Node: NodeNav + NodeData {}

impl<TNode: NodeData + NodeNav> Node for TNode {}

/// Information about the parent of a Node.
#[derive(Clone)]
pub struct ParentInfo<TNode> {
    pub node: TNode,
    pub label: Label,
}
