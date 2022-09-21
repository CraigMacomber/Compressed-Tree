//! Core types of the tree abstraction.

use crate::forest::util::ImSlice;

pub type IdBase = u128;

#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy)]
pub struct Def(pub IdBase);
#[derive(Clone, PartialEq, Eq, Ord, Hash, PartialOrd, Copy, Debug)]
pub struct Label(pub IdBase);

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

impl<'b, T> Indexable for &'b Vec<T> {
    type Item<'a> = &'a T where Self: 'a;

    fn index<'a>(&'a self, i: usize) -> Self::Item<'a> {
        self.get(i).unwrap()
    }
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

/// Navigation part of Node
pub trait NodeNav {
    /// For iterating children within a trait.
    type TTraitChildren<'a>: Indexable<Item<'a> = Self>
    where
        Self: 'a;
    /// For iterating the set of trait labels for non-empty traits.
    type TFields<'a>: Iterator<Item = (&'a Label, Self::TTraitChildren<'a>)>
    where
        Self: 'a;

    fn get_traits<'a>(&'a self) -> Self::TFields<'a>;
    fn get_trait<'a>(&'a self, label: Label) -> Self::TTraitChildren<'a>;
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
