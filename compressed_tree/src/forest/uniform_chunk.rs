use std::{rc::Rc, usize};

use super::{
    chunk::{HasId, NodeChunkIndex},
    tree::{Def, Indexable, Label, NodeData, NodeNav},
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
    /// Derived data (from schema) to enable fast lookup of views from id.
    id_offset_to_byte_offset_and_schema: Vec<Option<OffsetInfo>>,
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
        let mut data_outer = vec![None; schema.id_stride as usize];

        fn add(
            data: &mut [Option<OffsetInfo>],
            s: &ChunkSchema,
            byte_offset: u32,
            parent: ParentInfo,
        ) {
            for (label, sub_schema) in s.traits.iter() {
                for i in 0..sub_schema.schema.node_count {
                    add(
                        data,
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
            &mut data_outer.as_mut_slice(),
            &schema,
            0,
            ParentInfo {
                parent: None,
                index: 0,
            },
        );

        RootChunkSchema {
            schema,
            id_offset_to_byte_offset_and_schema: data_outer,
        }
    }
}

#[derive(Clone)]
pub struct ChunkSchema {
    pub def: Def,
    /// number of nodes at this level
    pub node_count: u32,
    pub bytes_per_node: u32,
    /// total number in subtree (nodes under traits + 1)
    pub id_stride: u32,
    pub payload_size: Option<u16>,
    pub traits: std::collections::HashMap<Label, OffsetSchema, ahash::RandomState>,
}

/// Offsets are for the first iteration (of a possible schema.node_count iterations)
/// and are relative to the immediate parent (the node not the trait).
/// Thus these offsets need to account for the parent's payload, the parent's id,
/// and all traits which precede this one (including their repetitions via node_count).
/// Note thats its allowed the layout in id space and byte space to differ, so which traits are preceding in each might not be the same.
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
    pub fn view(&self) -> ChunkInfo {
        ChunkInfo {
            schema: &self.schema.schema,
            data: self.data.focus(),
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

impl<'b> NodeNav for UniformChunkNode<'b> {
    type TTraitChildren<'a> = ChunkIndexer<'a> where Self: 'a;
    type TFields<'a> = ChunkFieldsIterator<'a> where Self: 'a;

    fn get_traits<'a>(&'a self) -> Self::TFields<'a> {
        todo!()
        // self.view.schema.traits.keys().cloned()
    }

    fn get_trait<'a>(&'a self, label: Label) -> Self::TTraitChildren<'a> {
        match self.view.schema.traits.get(&label) {
            Some(x) => {
                let node_data = self.data();
                let trait_data = slice_with_length(
                    node_data,
                    x.byte_offset as usize,
                    x.schema.bytes_per_node as usize,
                );
                let info: ChunkInfo<'a> = ChunkInfo {
                    schema: &x.schema,
                    data: trait_data,
                };
                ChunkIndexer::View(info)
            }
            None => ChunkIndexer::Empty,
        }
    }
}

// Views first item as chunk in as node
impl<'a> NodeData for UniformChunkNode<'a> {
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

// Views first item as chunk in as node
impl HasId for UniformChunkNode<'_> {
    fn get_index_in_chunk(&self) -> NodeChunkIndex {
        NodeChunkIndex(self.offset as usize * self.view.schema.id_stride as usize)
    }
}

pub enum ChunkIndexer<'a> {
    View(ChunkInfo<'a>),
    Empty,
}

impl<'b> Indexable for ChunkIndexer<'b> {
    type Item<'a> = UniformChunkNode<'a>;

    fn index<'a>(&'a self, index: usize) -> UniformChunkNode<'a> {
        match self {
            ChunkIndexer::View(ref info) => {
                if index < info.schema.node_count as usize {
                    UniformChunkNode {
                        view: info.clone(),
                        offset: index as u32,
                    }
                } else {
                    panic!()
                }
            }
            ChunkIndexer::Empty => panic!(),
        }
    }

    fn len(&self) -> usize {
        match self {
            ChunkIndexer::View(view) => view.schema.node_count as usize,
            ChunkIndexer::Empty => 0,
        }
    }
}

pub struct ChunkFieldsIterator<'a> {
    data: ImSlice<'a>,
    traits: std::collections::hash_map::Iter<'a, Label, OffsetSchema>,
}

impl<'a> Iterator for ChunkFieldsIterator<'a> {
    type Item = (&'a Label, ChunkIndexer<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (label, schema) = self.traits.next()?;
        let data = slice_with_length(
            self.data.clone(),
            schema.byte_offset as usize,
            schema.schema.bytes_per_node as usize,
        );
        let info: ChunkInfo = ChunkInfo {
            schema: &schema.schema,
            data: data,
        };

        Some((label, ChunkIndexer::View(info)))
    }
}
