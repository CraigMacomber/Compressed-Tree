use crate::{
    forest::tree::{Indexable, Node},
    EitherCursor, FieldKey, FieldsCursor, NodesCursor, TreeType, Value,
};

pub struct GenericFieldsCursor<'a, T: Node<'a>> {
    current: BasicCursorLevel<'a, T>,
    /// Cache of Nodes at the current key.
    nodes: T::TField,
    parents: Vec<BasicCursorLevel<'a, T>>,
}

struct BasicCursorLevel<'a, T: Node<'a>> {
    nodes: BasicCursorNodesLevel<'a, T>,
    fields: BasicCursorFieldsLevel<'a, T>,
}

struct BasicCursorNodesLevel<'a, T: Node<'a>> {
    index: usize,
    nodes: T::TField,
}

struct BasicCursorFieldsLevel<'a, T: Node<'a>> {
    key: FieldKey, // TODO: reference to some centralized Key object
    fields: Option<T::TFields>,
}

pub struct GenericNodesCursor<'a, T: Node<'a>> {
    current: BasicCursorNodesLevel<'a, T>,
    parents: Vec<BasicCursorLevel<'a, T>>,
}

impl<'a, T: Node<'a>> GenericNodesCursor<'a, T> {
    pub fn new(n: T::TField) -> GenericNodesCursor<'a, T> {
        GenericNodesCursor {
            parents: vec![],
            current: BasicCursorNodesLevel { index: 0, nodes: n },
        }
    }

    fn current_node(&self) -> T {
        self.current.nodes.index(self.current.index)
    }
}

impl<'a, T: Node<'a>> GenericNodesCursor<'a, T> {
    pub fn is_leaf(&self) -> bool {
        self.current_node().is_leaf()
    }
}

impl<'a, T: Node<'a>> NodesCursor for GenericNodesCursor<'a, T> {
    type TFields = GenericFieldsCursor<'a, T>;

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

    fn next_node(mut self) -> EitherCursor<Self, Self::TFields> {
        self.current.index += 1;
        if self.current.index < self.current.nodes.len() {
            EitherCursor::Nodes(self)
        } else {
            EitherCursor::Fields(self.exit_node())
        }
    }

    fn exit_node(mut self) -> Self::TFields {
        let current = self.parents.pop().unwrap();
        GenericFieldsCursor {
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

    fn first_field(self) -> EitherCursor<Self, Self::TFields> {
        let mut iter = self.current_node().get_fields();
        let first = iter.next();
        match first {
            Some((key, nodes)) => EitherCursor::Fields(GenericFieldsCursor {
                nodes,
                current: BasicCursorLevel {
                    nodes: self.current,
                    fields: BasicCursorFieldsLevel {
                        key: key.clone(),
                        fields: Some(iter),
                    },
                },
                parents: self.parents,
            }),
            None => EitherCursor::Nodes(self),
        }
    }

    fn enter_field(self, key: FieldKey) -> EitherCursor<Self, Self::TFields> {
        EitherCursor::Fields(GenericFieldsCursor {
            nodes: self.current_node().get_field(key.clone()),
            current: BasicCursorLevel {
                nodes: self.current,
                fields: BasicCursorFieldsLevel {
                    key: key.clone(),
                    fields: None,
                },
            },
            parents: self.parents,
        })
    }

    fn node_type(&self) -> TreeType {
        let def = self.current_node().get_def().clone();
        todo!()
        // TreeType(def)
    }
}

impl<'a, T: Node<'a>> FieldsCursor for GenericFieldsCursor<'a, T> {
    type TNodes = GenericNodesCursor<'a, T>;

    fn next_field(mut self) -> EitherCursor<Self::TNodes, Self> {
        let fields = &mut self.current.fields.fields;
        match fields {
            Some(f) => match f.next() {
                Some((key, nodes)) => {
                    self.current.fields.key = key.clone();
                    self.nodes = nodes;
                    EitherCursor::Fields(self)
                }
                None => EitherCursor::Nodes(self.exit_field()), // Was on last field.
            },
            None => EitherCursor::Nodes(self.exit_field()), // Used enter_field instead of iterating.
        }
    }

    fn exit_field(self) -> Self::TNodes {
        GenericNodesCursor {
            current: self.current.nodes,
            parents: self.parents,
        }
    }

    fn skip_pending_fields(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Fields(self)
    }

    fn get_field_length(&self) -> u32 {
        1
    }

    fn first_node(self) -> EitherCursor<Self::TNodes, Self> {
        if self.nodes.len() > 0 {
            EitherCursor::Nodes(self.enter_node(0))
        } else {
            EitherCursor::Fields(self)
        }
    }

    fn enter_node(mut self, child_index: u32) -> Self::TNodes {
        self.parents.push(self.current);
        GenericNodesCursor {
            current: BasicCursorNodesLevel {
                index: child_index as usize,
                nodes: self.nodes,
            },
            parents: self.parents,
        }
    }
}
