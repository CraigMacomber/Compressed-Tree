use std::{collections::HashMap, mem::replace, ops::DerefMut, rc::Rc};

use owning_ref::OwningHandle;
use wasm_bindgen::prelude::*;

use crate::{
    cursor::{GenericFieldsCursor, GenericNodesCursor},
    forest::{
        example_node::BasicNode,
        test_stuff::walk_all,
        tree::{Node, NodeNav},
        uniform_chunk::{
            ChunkSchema, OffsetSchema, RootChunkSchema, UniformChunk, UniformChunkNode,
        },
    },
    EitherCursor, FieldKey, FieldsCursor, NodesCursor, TreeType,
};

type InnerNode<'a> = UniformChunkNode<'a>;
type Tree = UniformChunk;
const BUILD_TEST_TREE: fn(usize, usize) -> Tree = chunked_test_tree;
// type InnerNode<'a> = &'a BasicNode;
// type Tree = Vec<BasicNode>;
// const BUILD_TEST_TREE: fn(usize, usize) -> Tree = basic_test_tree;

type StaticNode = InnerNode<'static>;
type Nodes<'a> = <InnerNode<'a> as NodeNav<'a>>::TField;

#[wasm_bindgen]
pub struct WasmCursor {
    data: Handle<StaticNode>,
}

struct CursorWrap<'a, T: Node<'a>>(Cursor<'a, T>);

enum Cursor<'a, T: Node<'a>> {
    Nodes(GenericNodesCursor<'a, T>),
    Fields(GenericFieldsCursor<'a, T>),
    Empty,
}

type Handle<T> = OwningHandle<Box<Tree>, CursorWrap<'static, T>>;

fn owning_handle(v: Tree) -> Handle<StaticNode> {
    let cell_ref = Box::new(v);
    let handle = OwningHandle::new_with_fn(cell_ref, |x| {
        let x = unsafe { x.as_ref() }.unwrap();
        // let y: Nodes = &x;
        let y: Nodes = x.view().view;
        let root = GenericNodesCursor::new(y);
        let cursor = CursorWrap(Cursor::Nodes(root));
        cursor
    });
    handle
}

// Not sure why OwningHandle requires this.
impl<'a, T: Node<'a>> core::ops::Deref for CursorWrap<'a, T> {
    type Target = Cursor<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Not sure why OwningHandle requires this.
impl<'a, T: Node<'a>> core::ops::DerefMut for CursorWrap<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl WasmCursor {
    fn cursor_mut(&mut self) -> &mut Cursor<'static, StaticNode> {
        self.data.deref_mut()
    }

    fn cursor(&self) -> &Cursor<'static, StaticNode> {
        &self.data
    }

    fn new(v: Tree) -> Self {
        WasmCursor {
            data: owning_handle(v),
        }
    }
}

fn basic_test_tree(fields: usize, per_field: usize) -> Vec<BasicNode> {
    fn test_node() -> BasicNode {
        let tree: BasicNode = BasicNode {
            def: TreeType("".into()),
            payload: None,
            fields: HashMap::default(),
        };
        tree
    }
    let mut root = test_node();
    for f in 0..fields {
        let children = (0..per_field).map(|_| test_node()).collect();
        root.fields.insert(FieldKey(f.to_string()), children);
    }
    vec![root]
}

fn chunked_test_tree(fields: usize, per_field: usize) -> UniformChunk {
    // Chunk of Leaf nodes schema
    let sub_schema = ChunkSchema {
        def: TreeType("".into()),
        node_count: per_field as u32,
        bytes_per_node: 0,
        payload_size: None,
        fields: HashMap::default(),
    };

    // Root schema
    let mut root = ChunkSchema {
        def: TreeType("".into()),
        node_count: (fields * per_field + 1) as u32,
        bytes_per_node: 0,
        payload_size: None,
        fields: HashMap::default(),
    };

    for f in 0..fields {
        let children = OffsetSchema {
            byte_offset: 0,
            schema: sub_schema.clone(),
        };
        root.fields.insert(FieldKey(f.to_string()), children);
    }

    let chunk_schema = Rc::new(RootChunkSchema::new(root));

    let data: Vec<u8> = vec![];
    debug_assert_eq!(data.len(), 0);

    UniformChunk {
        schema: chunk_schema.clone(),
        data,
    }
}

#[wasm_bindgen]
impl WasmCursor {
    /// Create a new tree of test data and a cursor over it.
    /// TODO: Public API for creating trees.
    #[wasm_bindgen(constructor)]
    pub fn new_from_test_data(fields: usize, per_field: usize) -> Self {
        WasmCursor::new(BUILD_TEST_TREE(fields, per_field))
    }

    #[wasm_bindgen(getter)]
    pub fn mode(&self) -> i32 {
        match self.cursor() {
            Cursor::Nodes(_) => 0,
            Cursor::Fields(_) => 1,
            Cursor::Empty => panic!(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn pending(&self) -> bool {
        // TODO
        false
    }

    #[wasm_bindgen(getter, js_name = fieldIndex)]
    pub fn field_index(&self) -> u32 {
        match &self.cursor() {
            Cursor::Nodes(n) => n.field_index(),
            _ => panic!(),
        }
    }

    #[wasm_bindgen(getter, js_name = chunkStart)]
    pub fn chunk_start(&self) -> u32 {
        match &self.cursor() {
            Cursor::Nodes(n) => n.chunk_start(),
            _ => panic!(),
        }
    }

    #[wasm_bindgen(getter, js_name = chunkLength)]
    pub fn chunk_length(&self) -> u32 {
        match &self.cursor() {
            Cursor::Nodes(n) => n.chunk_length(),
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = seekNodes)]
    pub fn seek_nodes(&mut self, offset: i32) -> bool {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => match n.seek_nodes(offset) {
                EitherCursor::Nodes(n) => {
                    *cursor = Cursor::Nodes(n);
                    true
                }
                EitherCursor::Fields(f) => {
                    *cursor = Cursor::Fields(f);
                    false
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = nextNode)]
    pub fn next_node(&mut self) -> bool {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => match n.next_node() {
                EitherCursor::Nodes(n) => {
                    *cursor = Cursor::Nodes(n);
                    true
                }
                EitherCursor::Fields(f) => {
                    *cursor = Cursor::Fields(f);
                    false
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = exitNode)]
    pub fn exit_node(&mut self) {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => {
                *cursor = Cursor::Fields(n.exit_node());
            }
            _ => panic!(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> Option<f64> {
        match &self.cursor() {
            Cursor::Nodes(n) => n.value().0,
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = firstField)]
    pub fn first_field(&mut self) -> bool {
        let cursor = self.cursor_mut();
        match cursor {
            Cursor::Nodes(n) => {
                if n.is_leaf() {
                    return false;
                }
            }
            _ => panic!(),
        }
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => match n.first_field() {
                EitherCursor::Nodes(n) => {
                    *cursor = Cursor::Nodes(n);
                    false
                }
                EitherCursor::Fields(f) => {
                    *cursor = Cursor::Fields(f);
                    true
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = enterField)]
    pub fn enter_field(&mut self, key: String) -> bool {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => match n.enter_field(FieldKey(key)) {
                EitherCursor::Nodes(n) => {
                    *cursor = Cursor::Nodes(n);
                    false
                }
                EitherCursor::Fields(f) => {
                    *cursor = Cursor::Fields(f);
                    true
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(getter, js_name = type)]
    pub fn node_type(&self) -> String {
        match &self.cursor() {
            Cursor::Nodes(n) => n.node_type().0,
            _ => panic!(),
        }
    }

    // ///////////////////////////

    #[wasm_bindgen(js_name = nextField)]
    pub fn next_field(&mut self) -> bool {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Fields(f) => match f.next_field() {
                EitherCursor::Nodes(n) => {
                    *cursor = Cursor::Nodes(n);
                    false
                }
                EitherCursor::Fields(f) => {
                    *cursor = Cursor::Fields(f);
                    true
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = exitField)]
    pub fn exit_field(&mut self) {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Fields(f) => {
                *cursor = Cursor::Nodes(f.exit_field());
            }
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = skipPendingFields)]
    pub fn skip_pending_fields(&mut self) -> bool {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Fields(f) => match f.skip_pending_fields() {
                EitherCursor::Nodes(n) => {
                    *cursor = Cursor::Nodes(n);
                    false
                }
                EitherCursor::Fields(f) => {
                    *cursor = Cursor::Fields(f);
                    true
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = getFieldLength)]
    pub fn get_field_length(&self) -> u32 {
        let cursor = self.cursor();
        match cursor {
            Cursor::Fields(f) => f.get_field_length(),
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = firstNode)]
    pub fn first_node(&mut self) -> bool {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Fields(f) => match f.first_node() {
                EitherCursor::Nodes(n) => {
                    *cursor = Cursor::Nodes(n);
                    true
                }
                EitherCursor::Fields(f) => {
                    *cursor = Cursor::Fields(f);
                    false
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = enterNode)]
    pub fn enter_node(&mut self, child_index: u32) {
        let cursor = self.cursor_mut();
        let old = replace(cursor, Cursor::Empty);
        match old {
            Cursor::Fields(f) => {
                *cursor = Cursor::Nodes(f.enter_node(child_index));
            }
            _ => panic!(),
        }
    }
}

/// Walks the subtree under the cursor's current node.
///
/// Returns the number of nodes in the subtree, including its root.
#[wasm_bindgen(js_name = walkSubtree)]
pub fn walk_subtree(n: &mut WasmCursor) -> usize {
    let mut count = 1;
    let mut in_fields = n.first_field();
    while in_fields {
        let mut in_nodes = n.first_node();
        while in_nodes {
            count += walk_subtree(n);
            in_nodes = n.next_node();
        }
        in_fields = n.next_field();
    }
    count
}

/// Walks the subtree under the cursor's current node.
///
/// Returns the number of nodes in the subtree, including its root.
#[wasm_bindgen(js_name = walkSubtreeDepth)]
pub fn walk_subtree_depth(n: &mut WasmCursor, depth: usize) -> usize {
    let mut count = 1;
    if depth > 0 {
        let mut in_fields = n.first_field();
        while in_fields {
            let mut in_nodes = n.first_node();
            while in_nodes {
                count += walk_subtree_depth(n, depth - 1);
                in_nodes = n.next_node();
            }
            in_fields = n.next_field();
        }
    }
    count
}

/// Walks the subtree under the cursor's current node.
/// Uses lower level API.
///
/// TODO:
/// For unknown reasons this is slower than the higher level API. Why?
///
/// Returns the number of nodes in the subtree, including its root.
#[wasm_bindgen(js_name = walkSubtreeInternal)]
pub fn walk_subtree_internal(n: &mut WasmCursor) -> usize {
    let cursor = n.cursor_mut();
    let old = replace(cursor, Cursor::Empty);
    let cursor_inner = match old {
        Cursor::Nodes(c) => c,
        _ => panic!(),
    };
    let mut count = 0;
    *cursor = Cursor::Nodes(inner(cursor_inner, &mut count));
    count
}

fn inner<'a, T: Node<'a>>(
    c: GenericNodesCursor<'a, T>,
    count: &mut usize,
) -> GenericNodesCursor<'a, T> {
    *count += 1;
    if c.is_leaf() {
        return c;
    }
    let mut in_fields = c.first_field();
    loop {
        match in_fields {
            EitherCursor::Nodes(n) => {
                return n;
            }
            EitherCursor::Fields(f) => {
                let mut in_nodes = f.first_node();
                loop {
                    match in_nodes {
                        EitherCursor::Nodes(n) => {
                            in_nodes = inner(n, count).next_node();
                        }
                        EitherCursor::Fields(f) => {
                            in_fields = f.next_field();
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Walks the tree this cursor is attached to.
/// Uses even lower level API.
///
/// Returns the number of nodes in the tree, including its root.
#[wasm_bindgen(js_name = walkSubtreeInternal2)]
pub fn walk_subtree_internal2(n: &mut WasmCursor) -> usize {
    let tree = n.data.as_owner();
    // walk_all(&tree[0])
    walk_all(tree.view())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_wasm_cursor() {
        let mut cursor = WasmCursor::new_from_test_data(10, 10);
        assert_eq!(walk_subtree(&mut cursor), 101);
    }

    #[test]
    fn walk_wasm_cursor_internal() {
        let mut cursor = WasmCursor::new_from_test_data(10, 10);
        assert_eq!(walk_subtree_internal(&mut cursor), 101);
    }

    #[test]
    fn walk_wasm_cursor_internal2() {
        let mut cursor = WasmCursor::new_from_test_data(10, 10);
        assert_eq!(walk_subtree_internal2(&mut cursor), 101);
    }
}
