use super::{
    tree::{Def, Label, Node, NodeNav, Indexable},
    uniform_chunk::{ChunkSchema, OffsetSchema, RootChunkSchema, UniformChunk},
};
use rand::Rng;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub fn big_tree(chunk_size: usize) -> UniformChunk {
    let rng = RefCell::new(rand::thread_rng());
    let new_label = || -> Label { Label(rng.borrow_mut().gen()) };
    let new_def = || -> Def { Def(rng.borrow_mut().gen()) };
    let def = new_def();
    let label = new_label();

    // color channel schema
    let sub_schema = ChunkSchema {
        def: new_def(),
        node_count: 1,
        bytes_per_node: 1,
        payload_size: Some(1),
        traits: HashMap::default(),
    };

    // Color schema (rgba)
    let schema = ChunkSchema {
        def: new_def(),
        node_count: chunk_size as u32,
        bytes_per_node: 4,
        payload_size: None,
        traits: vec![
            (
                new_label(),
                OffsetSchema {
                    byte_offset: 0,
                    schema: sub_schema.clone(),
                },
            ),
            (
                new_label(),
                OffsetSchema {
                    byte_offset: 1,
                    schema: sub_schema.clone(),
                },
            ),
            (
                new_label(),
                OffsetSchema {
                    byte_offset: 2,
                    schema: sub_schema.clone(),
                },
            ),
            (
                new_label(),
                OffsetSchema {
                    byte_offset: 3,
                    schema: sub_schema,
                },
            ),
        ]
        .into_iter()
        .collect(),
    };

    let chunk_schema = Rc::new(RootChunkSchema::new(schema));

    let data: im_rc::Vector<u8> = std::iter::repeat(&[1u8, 2, 3, 4])
        .take(chunk_size)
        .flat_map(|x| x.iter())
        .cloned()
        .collect();
    debug_assert_eq!(data.len(), chunk_size * 4);

    UniformChunk {
        schema: chunk_schema.clone(),
        data: data.into(),
    }
}

pub fn walk_all<'a, T>(n: &'a T) -> usize where
    T: Node,
    T: NodeNav<TTraitChildren<'a>: Indexable<Item<'a>= T>>,
    {
    let mut count = 1;
    for (_, t) in n.get_traits() {
        for c in 0..t.len() {
            let child = t.index(c);
            count += walk_all(&child);
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_nodes() {
        let chunk = big_tree(1000);
        // assert_eq!(walk_all(nav), size);
    }

    #[test]
    fn print_sizes() {
        println!("UniformChunk:{}", std::mem::size_of::<UniformChunk>(),);
        // panic!();
    }
}
