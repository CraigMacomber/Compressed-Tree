//! Simple tree that owns its children.
//! This serves as an example of the simplest way to implement Node, and is not actually used.

use std::collections::HashMap;

use crate::{TreeType, FieldKey};

use super::{
    tree::{NodeData, NodeNav},
    util::ImSlice,
};

pub struct BasicNode {
    pub def: TreeType,
    pub payload: Option<im_rc::Vector<u8>>,
    pub fields: HashMap<FieldKey, Vec<BasicNode>>, // TODO: Use hash map from im_rc
}

impl<'a> NodeNav<'a> for &'a BasicNode {
    type TField = &'a [BasicNode];
    type TFields = FieldIterator<'a>;

    fn get_field(&self, label: FieldKey) -> Self::TField {
        self.fields.get(&label).unwrap_or(EMPTY)
    }
    fn get_fields(&self) -> Self::TFields {
        FieldIterator {
            data: self.fields.iter(),
        }
    }
}

impl<'b> NodeData for &'b BasicNode {
    fn get_def(&self) -> TreeType {
        self.def.clone() // TODO
    }

    fn get_payload(&self) -> Option<ImSlice> {
        self.payload.as_ref().map(|p| p.focus())
    }
}

const EMPTY: &Vec<BasicNode> = &vec![];

pub type BasicTree<'a> = &'a BasicNode;

pub struct FieldIterator<'a> {
    data: std::collections::hash_map::Iter<'a, FieldKey, Vec<BasicNode>>,
}

impl<'a> Iterator for FieldIterator<'a> {
    type Item = (&'a FieldKey, &'a [BasicNode]);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, vec) = self.data.next()?;
        Some((key, vec))
    }
}
