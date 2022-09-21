//! Simple tree that owns its children.
//! This serves as an example of the simplest way to implement Node, and is not actually used.

use std::collections::HashMap;

use super::{
    tree::{Def, Label, NodeData, NodeNav},
    util::ImSlice,
};

pub struct BasicNode {
    pub def: Def,
    pub payload: Option<im_rc::Vector<u8>>,
    pub traits: HashMap<Label, Vec<BasicNode>>, // TODO: Use hash map from im_rc
}

impl<'b> NodeNav for &'b BasicNode {
    type TTraitChildren<'a> = &'a Vec<BasicNode> where Self: 'a;
    type TFields<'a> = std::collections::hash_map::Iter<'a, Label, Vec<BasicNode>> where Self: 'a;

    fn get_traits<'a>(&'a self) -> Self::TFields<'a> {
        self.traits.iter()
    }

    fn get_trait<'a>(&'a self, label: Label) -> Self::TTraitChildren<'a> {
        self.traits.get(&label).unwrap_or(EMPTY)
    }
}

impl<'b> NodeData for &'b BasicNode {
    fn get_def(&self) -> Def {
        self.def
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.payload.as_ref().map(|p| p.focus())
    }
}

const EMPTY: &Vec<BasicNode> = &vec![];
