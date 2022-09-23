use std::{collections::HashMap, rc::Rc, usize};

use crate::{FieldKey, TreeType};

use super::{
    tree::{Indexable, NodeData, NodeNav, Tree},
    util::{slice_with_length, ImSlice},
};

/// Sequence of trees with identical schema and sequential ids (depth first pre-order).
/// Owns the content. Compressed (one copy of schema, rest as blob)
#[derive(Clone)]
pub struct UniformChunk {
    data: Vec<u8>,
    schema: Rc<ChunkSchema>,
}

impl PartialEq for UniformChunk {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.schema, &other.schema) & self.data.eq(&other.data)
    }
}

#[derive(Clone)]
pub struct ChunkSchema {
    pub def: TreeType,
    /// number of nodes at this level
    pub node_count: u32,
    pub bytes_per_node: u32,
    pub payload_size: Option<u16>,
    pub fields: std::collections::HashMap<FieldKey, OffsetSchema, ahash::RandomState>,
}

/// Offsets are for the first iteration (of a possible schema.node_count iterations)
/// and are relative to the immediate parent (the node not the field).
/// Thus these offsets need to account for the parent's payload, the parent's id,
/// and all fields which precede this one (including their repetitions via node_count).
/// Note thats its allowed the layout in id space and byte space to differ, so which fields are preceding in each might not be the same.
/// Its also allowed to leave unused gaps in either id space or byte space.
#[derive(Clone)]
pub struct OffsetSchema {
    pub byte_offset: u32,
    pub schema: ChunkSchema,
}

// Views

/// Info about part of a chunk.
#[derive(Clone)]
pub struct ChunkInfo<'a> {
    schema: &'a ChunkSchema,
    data: ImSlice<'a>,
}

/// Node within a [UniformChunk]
#[derive(Clone)]
pub struct UniformChunkNode<'a> {
    pub view: ChunkInfo<'a>, // the field this node is in
    pub offset: u32,         // index of current node in its containing field
}

impl UniformChunk {
    pub fn new(schema: Rc<ChunkSchema>, data: Vec<u8>) -> UniformChunk {
        debug_assert_eq!(
            schema.bytes_per_node as usize * schema.node_count as usize,
            data.len()
        );
        UniformChunk { schema, data }
    }

    pub fn get_count(&self) -> usize {
        self.schema.node_count as usize
    }
}

impl Tree for UniformChunk {
    type TNode<'a> = UniformChunkNode<'a>;

    fn view(&self) -> ChunkInfo {
        ChunkInfo {
            schema: &self.schema,
            data: self.data.as_slice(),
        }
    }
}

impl<'a> UniformChunkNode<'a> {
    fn data(&self) -> ImSlice<'a> {
        let offset = self.offset as usize;
        let stride = self.view.schema.bytes_per_node as usize;
        let start = offset * stride;
        slice_with_length(self.view.data, start, stride)
    }
}

impl<'a> NodeNav<'a> for UniformChunkNode<'a> {
    type TField = ChunkInfo<'a>;
    type TFields = ChunkFieldsIterator<'a>;

    fn get_field(&self, label: FieldKey) -> Self::TField {
        match self.view.schema.fields.get(&label) {
            Some(x) => {
                let node_data = self.data();
                let field_data = slice_with_length(
                    node_data,
                    x.byte_offset as usize,
                    x.schema.bytes_per_node as usize,
                );
                ChunkInfo {
                    schema: &x.schema,
                    data: field_data,
                }
            }
            None => ChunkInfo {
                schema: &EMPTY_SCHEMA,
                data: slice_with_length(self.data(), 0, 0),
            },
        }
    }

    fn get_fields(&self) -> Self::TFields {
        ChunkFieldsIterator {
            data: self.data(),
            fields: self.view.schema.fields.iter(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.view.schema.fields.len() == 0
    }
}

// Views first item as chunk in as node
impl NodeData for UniformChunkNode<'_> {
    fn get_def(&self) -> TreeType {
        self.view.schema.def.clone() // TODO
    }

    fn get_payload(&self) -> Option<ImSlice> {
        match self.view.schema.payload_size {
            Some(p) => {
                let node_data = self.data();
                Some(slice_with_length(node_data, 0, p as usize))
            }
            None => None,
        }
    }
}

impl<'a> Indexable for ChunkInfo<'a> {
    type Item = Option<UniformChunkNode<'a>>;

    fn index(&self, index: usize) -> Self::Item {
        if index < self.schema.node_count as usize {
            Some(UniformChunkNode {
                view: self.clone(),
                offset: index as u32,
            })
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.schema.node_count as usize
    }
}

pub struct ChunkFieldsIterator<'a> {
    data: ImSlice<'a>,
    fields: std::collections::hash_map::Iter<'a, FieldKey, OffsetSchema>,
}

impl<'a> Iterator for ChunkFieldsIterator<'a> {
    type Item = (&'a FieldKey, ChunkInfo<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (label, schema) = self.fields.next()?;
        let data = slice_with_length(
            self.data,
            schema.byte_offset as usize,
            schema.schema.bytes_per_node as usize,
        );
        let info: ChunkInfo = ChunkInfo {
            schema: &schema.schema,
            data: data,
        };

        Some((label, info))
    }
}

lazy_static! {
    static ref EMPTY_SCHEMA: ChunkSchema = ChunkSchema {
        def: TreeType("".into()),
        node_count: 0,
        bytes_per_node: 0,
        payload_size: None,
        fields: HashMap::default(),
    };
}
