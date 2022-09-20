use crate::{NodesCursor, EitherCursor, Value, FieldKey, TreeType, FieldsCursor};

pub struct BasicNodes {}

impl NodesCursor for BasicNodes {
    type TFields = BasicFields;

    fn field_index(&self) -> i32 {
        0
    }

    fn chunk_start(&self) -> i32 {
        self.field_index()
    }

    fn chunk_length(&self) -> i32 {
        1
    }

    fn seek_nodes(self, offset: i32) -> EitherCursor<Self, Self::TFields> {
        EitherCursor::Nodes(self)
    }

    fn next_node(self) -> EitherCursor<Self, Self::TFields> {
        self.seek_nodes(1)
    }

    fn exit_node(self) -> Self::TFields {
        BasicFields{}
    }


    fn value(&self) -> Value {
        Value(Some(42f64))
    }

    fn first_field(self) -> EitherCursor<Self, Self::TFields> {
        EitherCursor::Fields(BasicFields{})
    }

    fn enter_field(self, key: FieldKey) -> EitherCursor<Self, Self::TFields> {
        EitherCursor::Fields(BasicFields{})
    }

    fn node_type(&self) -> TreeType {
        TreeType("TODO".into())
    }
}

pub struct BasicFields {}

impl FieldsCursor for BasicFields {
    type TNodes = BasicNodes;

    fn next_field(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Nodes(BasicNodes{})
    }

    fn exit_field(self) -> Self::TNodes {
        BasicNodes{}
    }

    fn skip_pending_fields(self) -> EitherCursor<Self::TNodes, Self>  {
        EitherCursor::Fields(self)
    }

    fn get_field_length(&self) -> i32 {
        1
    }

    fn first_node(self) -> EitherCursor<Self::TNodes, Self> {
        EitherCursor::Nodes(BasicNodes{})
    }

    fn enter_node(self, child_index: i32) -> Self::TNodes {
        BasicNodes{}
    }
}
