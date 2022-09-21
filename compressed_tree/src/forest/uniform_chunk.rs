use std::{collections::HashMap, rc::Rc, usize};

use super::{
    tree::{Def, Indexable, Label, NodeData, NodeNav, FieldMap},
    util::{slice_with_length, ImSlice},
};

/// Sequence of trees with identical schema and sequential ids (depth first pre-order).
/// Owns the content. Compressed (one copy of schema, rest as blob)
#[derive(Clone)]
pub struct UniformChunk {
    pub data: Box<im_rc::Vector<u8>>,
    pub schema: Rc<RootChunkSchema>,
}

impl PartialEq for UniformChunk {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.schema, &other.schema) & self.data.eq(&other.data)
    }
}

pub struct RootChunkSchema {
    pub schema: ChunkSchema,
}

#[derive(Clone)]
struct OffsetInfo {
    byte_offset: u32,
    schema: ChunkSchema,
    parent: ParentInfo,
}

#[derive(Clone)]
pub struct ParentInfo {
    /// None for top level nodes in chunk
    pub parent: Option<Label>,
    pub index: usize,
}

#[derive(Clone)]
pub struct OffsetInfoRef<'a> {
    pub byte_offset: u32,
    pub schema: &'a ChunkSchema,
    pub parent: ParentInfo,
}

impl RootChunkSchema {
    pub fn new(schema: ChunkSchema) -> Self {
        fn add(s: &ChunkSchema, byte_offset: u32, parent: ParentInfo) {
            // TODO: compute any desired derived data from schema here.
            for (label, sub_schema) in s.fields.iter() {
                for i in 0..sub_schema.schema.node_count {
                    add(
                        &sub_schema.schema,
                        byte_offset + sub_schema.byte_offset + i * sub_schema.schema.bytes_per_node,
                        ParentInfo {
                            parent: Some(*label),
                            index: i as usize,
                        },
                    )
                }
            }
        }

        add(
            &schema,
            0,
            ParentInfo {
                parent: None,
                index: 0,
            },
        );

        RootChunkSchema { schema }
    }
}

#[derive(Clone)]
pub struct ChunkSchema {
    pub def: Def,
    /// number of nodes at this level
    pub node_count: u32,
    pub bytes_per_node: u32,
    pub payload_size: Option<u16>,
    pub fields: std::collections::HashMap<Label, OffsetSchema, ahash::RandomState>,
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
    pub fn get_count(&self) -> usize {
        self.schema.schema.node_count as usize
    }

    /// View the first node in the chunk.
    /// TODO: return an iterator over chunk instead
    pub fn view(&self) -> UniformChunkNode {
        UniformChunkNode {
            view: ChunkInfo {
                schema: &self.schema.schema,
                data: self.data.focus(),
            },
            offset: 0,
        }
    }
}

impl<'a> UniformChunkNode<'a> {
    fn data(&self) -> ImSlice<'a> {
        let offset = self.offset as usize;
        let stride = self.view.schema.bytes_per_node as usize;
        let start = offset * stride;
        slice_with_length(self.view.data.clone(), start, stride)
    }
}

impl FieldMap for UniformChunkNode<'_> {
    type TField<'a> = ChunkInfo<'a> where Self: 'a;

    fn get_field(&self, label: Label) -> Self::TField<'_> {
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
}

impl NodeNav for UniformChunkNode<'_> {
    type TFields<'a> = ChunkFieldsIterator<'a> where Self: 'a;

    fn get_fields<'a>(&'a self) -> Self::TFields<'a> {
        ChunkFieldsIterator{ data: self.data(), fields: self.view.schema.fields.iter()}
    }
}

// Views first item as chunk in as node
impl NodeData for UniformChunkNode<'_> {
    fn get_def(&self) -> Def {
        self.view.schema.def
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

impl<'b> Indexable for ChunkInfo<'b> {
    type Item<'a> = UniformChunkNode<'a>;

    fn index<'a>(&'a self, index: usize) -> UniformChunkNode<'a> {
        if index < self.schema.node_count as usize {
            UniformChunkNode {
                view: self.clone(),
                offset: index as u32,
            }
        } else {
            panic!()
        }
    }

    fn len(&self) -> usize {
        self.schema.node_count as usize
    }
}

pub struct ChunkFieldsIterator<'a> {
    data: ImSlice<'a>,
    fields: std::collections::hash_map::Iter<'a, Label, OffsetSchema>,
}

impl<'a> Iterator for ChunkFieldsIterator<'a> {
    type Item = (&'a Label, ChunkInfo<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        // let EMPTY_DATA: im_rc::vector::Vector<u8> = im_rc::vector::Vector::default();
        let (label, schema) = self.fields.next()?;
        let data = slice_with_length(
            self.data.clone(),
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
        def: Def(0),
        node_count: 0,
        bytes_per_node: 0,
        payload_size: None,
        fields: HashMap::default(),
    };
}
