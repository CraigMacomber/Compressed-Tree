use std::{collections::HashMap, mem::replace, ops::DerefMut};

use owning_ref::OwningHandle;
use wasm_bindgen::prelude::*;

use crate::{
    basic_tree::{BasicFieldsCursor, BasicNodesCursor},
    forest::{example_node::BasicNode, tree::Node},
    EitherCursor, FieldKey, FieldsCursor, NodesCursor, TreeType,
};

#[wasm_bindgen]
pub struct WasmCursor {
    data: Handle<&'static BasicNode>,
}

struct CursorWrap<'a, T: Node<'a>>(Cursor<'a, T>);

enum Cursor<'a, T: Node<'a>> {
    Nodes(BasicNodesCursor<'a, T>),
    Fields(BasicFieldsCursor<'a, T>),
    Empty,
}

type Handle<T> = OwningHandle<Box<Vec<BasicNode>>, CursorWrap<'static, T>>;

fn owning_handle(v: Vec<BasicNode>) -> Handle<&'static BasicNode> {
    let cell_ref = Box::new(v);
    let handle = OwningHandle::new_with_fn(cell_ref, |x| {
        let x = unsafe { x.as_ref() }.unwrap();
        let y: &[BasicNode] = &x;
        let root = BasicNodesCursor::new(y);
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
    fn cursor_mut(&mut self) -> &mut Cursor<'static, &'static BasicNode> {
        self.data.deref_mut()
    }

    fn cursor(&self) -> &Cursor<'static, &'static BasicNode> {
        &self.data
    }

    fn new(v: Vec<BasicNode>) -> Self {
        WasmCursor {
            data: owning_handle(v),
        }
    }
}

#[wasm_bindgen]
impl WasmCursor {
    /// Create a new tree of test data and a cursor over it.
    /// TODO: Public API for creating trees.
    #[wasm_bindgen(constructor)]
    pub fn new_from_test_data(fields: usize, per_field: usize) -> Self {
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
        WasmCursor::new(vec![root])
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
        self.seek_nodes(1)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_wasm_cursor() {
        let mut cursor = WasmCursor::new_from_test_data(10, 10);
        assert_eq!(walk_subtree(&mut cursor), 101);
    }
}
