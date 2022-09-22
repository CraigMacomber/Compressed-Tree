use crate::{EitherCursor, FieldKey, FieldsCursor, NodesCursor, TreeType, Value};

pub struct DummyNodes {}

impl NodesCursor for DummyNodes {
    type TFields = DummyFields;

    fn field_index(&self) -> u32 {
        0
    }

    fn chunk_start(&self) -> u32 {
        self.field_index()
    }

    fn chunk_length(&self) -> u32 {
        1
    }

    fn seek_nodes(self, _offset: i32) -> EitherCursor<Self, Self::TFields> {
        EitherCursor::Nodes(self)
    }

    fn next_node(self) -> EitherCursor<Self, Self::TFields> {
        self.seek_nodes(1)
    }

    fn exit_node(self) -> Self::TFields {
        DummyFields {}
    }

    fn value(&self) -> Value {
        Value(Some(42f64))
    }

    fn first_field(self) -> EitherCursor<Self, Self::TFields> {
        EitherCursor::Fields(DummyFields {})
    }

    fn enter_field(self, _key: FieldKey) -> EitherCursor<Self, Self::TFields> {
        EitherCursor::Fields(DummyFields {})
    }

    fn node_type(&self) -> TreeType {
        TreeType("TODO".into())
    }
}

pub struct DummyFields {}

impl FieldsCursor for DummyFields {
    type TNodes = DummyNodes;

    fn next_field(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Nodes(DummyNodes {})
    }

    fn exit_field(self) -> Self::TNodes {
        DummyNodes {}
    }

    fn skip_pending_fields(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Fields(self)
    }

    fn get_field_length(&self) -> u32 {
        1
    }

    fn first_node(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Nodes(DummyNodes {})
    }

    fn enter_node(self, _child_index: u32) -> Self::TNodes {
        DummyNodes {}
    }
}
