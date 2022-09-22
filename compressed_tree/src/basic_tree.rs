use owning_ref::OwningRef;

use crate::{
    forest::{
        example_node::{BasicNode, FieldIterator},
        tree::{FieldMap, Indexable, Label, NodeNav},
    },
    EitherCursor, FieldKey, FieldsCursor, NodesCursor, TreeType, Value,
};

pub type BasicFields<'a> = FieldIterator<'a>;
pub type BasicNodes<'a> = &'a [BasicNode];

pub struct BasicFieldsCursor<'a> {
    current: BasicCursorLevel<'a>,
    /// Cache of Nodes at the current key.
    nodes: BasicNodes<'a>,
    parents: Vec<BasicCursorLevel<'a>>,
}

struct BasicCursorLevel<'a> {
    nodes: BasicCursorNodesLevel<'a>,
    fields: BasicCursorFieldsLevel<'a>,
}

struct BasicCursorNodesLevel<'a> {
    index: usize,
    nodes: BasicNodes<'a>,
}

struct BasicCursorFieldsLevel<'a> {
    key: &'a Label,
    fields: BasicFields<'a>,
}

pub struct BasicNodesCursor<'a> {
    current: BasicCursorNodesLevel<'a>,
    parents: Vec<BasicCursorLevel<'a>>,
}

pub struct BasicNodesCursor2 {
    current: BasicCursorNodesLevel<'static>,
    parents: Vec<BasicCursorLevel<'static>>,
}

// pub fn from_root(n: BasicNode) -> OwningRef<Vec<BasicNode>, BasicNodesCursor<'static>> {
//     let v = vec![n];
//     BasicNodesCursor{
//         parents: vec![],
//         current: BasicCursorNodesLevel{ index: 0, nodes: &v },
//     }
// }

pub fn from_root(n: BasicNode) -> OwningRef<Vec<BasicNode>, BasicNodesCursor<'static>> {
    let v = vec![n];
    let or = OwningRef::new(v);
    let or = or.map(|v| &BasicNodesCursor {
        parents: vec![],
        current: BasicCursorNodesLevel { index: 0, nodes: v },
    });
    or
}

impl<'a> BasicNodesCursor<'a> {
    fn current_node(&'a self) -> &'a BasicNode {
        self.current.nodes.index(self.current.index)
    }
}

impl<'a> NodesCursor for BasicNodesCursor<'a> {
    type TFields = BasicFieldsCursor<'a>;

    fn field_index(&self) -> u32 {
        self.current.index as u32
    }

    fn chunk_start(&self) -> u32 {
        self.field_index()
    }

    fn chunk_length(&self) -> u32 {
        1
    }

    fn seek_nodes(mut self, offset: i32) -> EitherCursor<Self, Self::TFields> {
        // TODO: correct over/underflow handling.
        let index = self.current.index as isize + offset as isize;
        if index < 0 || (index as usize) >= self.current.nodes.len() {
            EitherCursor::Fields(self.exit_node())
        } else {
            self.current.index = index as usize;
            EitherCursor::Nodes(self)
        }
    }

    fn next_node(self) -> EitherCursor<Self, Self::TFields> {
        self.seek_nodes(1)
    }

    fn exit_node(mut self) -> Self::TFields {
        let current = self.parents.pop().unwrap();
        BasicFieldsCursor {
            nodes: self.current.nodes,
            current,
            parents: self.parents,
        }
    }

    fn value(&self) -> Value {
        let node = self.current_node();
        todo!()
        // Value(self.current.get(self.index).map(|n| n.get_payload()))
    }

    fn first_field(mut self) -> EitherCursor<Self, Self::TFields> {
        let mut iter: FieldIterator<'a> = self.current_node().get_fields();
        let first = iter.next();
        match first {
            Some((key, nodes)) => EitherCursor::Fields(BasicFieldsCursor {
                nodes,
                current: BasicCursorLevel {
                    nodes: self.current,
                    fields: BasicCursorFieldsLevel {
                        key,
                        fields: iter,
                    },
                },
                parents: self.parents,
            }),
            None => EitherCursor::Nodes(self),
        }
    }

    fn enter_field(self, key: FieldKey) -> EitherCursor<Self, Self::TFields> {
        let label: Label = key;
        EitherCursor::Fields(BasicFieldsCursor {
            nodes: self.current_node().get_field(label),
            current: todo!(),
            parents: self.parents,
        })
    }

    fn node_type(&self) -> TreeType {
        let def = self.current_node().def.clone();
        todo!()
        // TreeType(def)
    }
}

impl<'a> FieldsCursor for BasicFieldsCursor<'a> {
    type TNodes = BasicNodesCursor<'a>;

    fn next_field(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Nodes(BasicNodesCursor {
            current: todo!(),
            parents: todo!(),
        })
    }

    fn exit_field(self) -> Self::TNodes {
        BasicNodesCursor {
            current: todo!(),
            parents: self.parents,
        }
    }

    fn skip_pending_fields(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Fields(self)
    }

    fn get_field_length(&self) -> i32 {
        1
    }

    fn first_node(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Nodes(BasicNodesCursor {
            current: todo!(),
            parents: todo!(),
        })
    }

    fn enter_node(self, child_index: i32) -> Self::TNodes {
        BasicNodesCursor {
            current: todo!(),
            parents: todo!(),
        }
    }
}
