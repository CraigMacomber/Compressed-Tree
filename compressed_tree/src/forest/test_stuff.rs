use super::{
    tree::{Def, Indexable, Label, Node},
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


pub fn walk_all<T: Node>(n: T) -> usize
{
    let mut count = 1;
    for (_, t) in n.get_traits() {
        for c in 0..t.len() {
            let child = t.index(c);
            count += walk_all(child);
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use crate::forest::{example_node::BasicNode, tree::NodeNav};

    use super::*;

    #[test]
    fn walk_chunk() {
        let chunk: UniformChunk = big_tree(1000);
        let view = chunk.view();

        assert_eq!(walk_all(view), 4000);
    }

    #[test]
    fn walk_basic() {
        let n: &BasicNode = &BasicNode {
            def: Def(0),
            payload: None,
            traits: HashMap::default(),
        };

        assert_eq!(walk_all(n), 1);
    }

    #[test]
    fn basic_nodes3() {
        let n: &BasicNode = &BasicNode {
            def: Def(0),
            payload: None,
            traits: HashMap::default(),
        };
        let field = n.get_trait(Label(0));
        for c in 0..field.len() {
            let child = field.index(c);
            let field2 = child.get_trait(Label(0));
            for c in 0..field2.len() {
                let child2 = field.index(c);
            }
        }
    }

    #[test]
    fn basic_nodes4() {
        let n: &BasicNode = &BasicNode {
            def: Def(0),
            payload: None,
            traits: HashMap::default(),
        };
        for (l, field) in n.get_traits() {
            for c in 0..field.len() {
                let child = field.index(c);
                for (l, field2) in n.get_traits() {
                    for c in 0..field2.len() {
                        let child2 = field.index(c);
                    }
                }
            }
        }
    }

    #[test]
    fn print_sizes() {
        println!("UniformChunk:{}", std::mem::size_of::<UniformChunk>(),);
        // panic!();
    }
}
