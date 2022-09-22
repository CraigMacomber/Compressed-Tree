use std::{
     collections::HashMap, mem::replace, ops::DerefMut,
};

use owning_ref::{OwningHandle};
use wasm_bindgen::prelude::*;

use crate::{
    basic_tree::{
        from_root,  BasicFieldsCursor,
        BasicNodesCursor,
    },
    forest::example_node::BasicNode,
    EitherCursor, NodesCursor, TreeType,
};

#[wasm_bindgen]
pub struct WasmCursor {
    data: Handle,
}

struct CursorWrap<'a>(Cursor<'a>);

enum Cursor<'a> {
    Nodes(BasicNodesCursor<'a>),
    Fields(BasicFieldsCursor<'a>),
    Empty,
}

type Handle = OwningHandle<Box<Vec<BasicNode>>, CursorWrap<'static>>;

fn owning_handle(n: BasicNode) -> Handle {
    let v = vec![n];
    let cell_ref = Box::new(v);
    let handle = OwningHandle::new_with_fn(cell_ref, |x| {
        let x = unsafe { x.as_ref() }.unwrap();
        let y: &[BasicNode] = &x;
        let root = from_root(y);
        let cursor = CursorWrap(Cursor::Nodes(root));
        cursor
    });
    handle
}

// Not sure why OwningHandle requires this.
impl<'a> core::ops::Deref for CursorWrap<'a> {
    type Target = Cursor<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Not sure why OwningHandle requires this.
impl<'a> core::ops::DerefMut for CursorWrap<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl WasmCursor {
    fn cursor_mut(&mut self) -> &mut Cursor<'static> {
        self.data.deref_mut()
    }

    fn cursor(&self) -> &Cursor<'static> {
        &self.data
    }
}

#[wasm_bindgen]
impl WasmCursor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let tree: BasicNode = BasicNode {
            def: TreeType("".into()),
            payload: None,
            fields: HashMap::default(),
        };

        WasmCursor {
            data: owning_handle(tree),
        }
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
                    replace(cursor, Cursor::Nodes(n));
                    true
                }
                EitherCursor::Fields(f) => {
                    replace(cursor, Cursor::Fields(f));
                    false
                }
            },
            _ => panic!(),
        }
    }

    // #[wasm_bindgen(js_name = nextNode)]
    // pub fn next_node(&mut self) -> bool {
    //     self.seek_nodes(1)
    // }

    // #[wasm_bindgen(js_name = exitNode)]
    // pub fn exit_node(&mut self) {
    //     let old = replace(&mut self.cursor, Cursor::Empty);
    //     match old {
    //         Cursor::Nodes(n) => {
    //             self.cursor = Cursor::Fields(n.exit_node());
    //         }
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(getter)]
    // pub fn value(&self) -> Option<f64> {
    //     match &self.cursor {
    //         Cursor::Nodes(n) => n.value().0,
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(js_name = firstField)]
    // pub fn first_field(&mut self) -> bool {
    //     let old = replace(&mut self.cursor, Cursor::Empty);
    //     match old {
    //         Cursor::Nodes(n) => match n.first_field() {
    //             EitherCursor::Nodes(n) => {
    //                 self.cursor = Cursor::Nodes(n);
    //                 false
    //             }
    //             EitherCursor::Fields(f) => {
    //                 self.cursor = Cursor::Fields(f);
    //                 true
    //             }
    //         },
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(js_name = enterField)]
    // pub fn enter_field(&mut self, key: String) -> bool {
    //     let old = replace(&mut self.cursor, Cursor::Empty);
    //     match old {
    //         Cursor::Nodes(n) => {
    //             match n.enter_field(FieldKey(key)) {
    //                 EitherCursor::Nodes(n) => {
    //                     self.cursor = Cursor::Nodes(n);
    //                     false
    //                 },
    //                 EitherCursor::Fields(f) => {
    //                     self.cursor = Cursor::Fields(f);
    //                     true
    //                 },
    //             }
    //         }
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(getter, js_name = type)]
    // pub fn node_type(&self) -> String {
    //     match &self.cursor {
    //         Cursor::Nodes(n) => n.node_type().0,
    //         _ => panic!(),
    //     }
    // }

    // ///////////////////////////

    // #[wasm_bindgen(js_name = nextField)]
    // pub fn next_field(&mut self) -> bool {
    //     let old = replace(&mut self.cursor, Cursor::Empty);
    //     match old {
    //         Cursor::Fields(f) => match f.next_field() {
    //             EitherCursor::Nodes(n) => {
    //                 self.cursor = Cursor::Nodes(n);
    //                 false
    //             }
    //             EitherCursor::Fields(f) => {
    //                 self.cursor = Cursor::Fields(f);
    //                 true
    //             }
    //         },
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(js_name = exitField)]
    // pub fn exit_field(&mut self) {
    //     let old = replace(&mut self.cursor, Cursor::Empty);
    //     match old {
    //         Cursor::Fields(f) => {
    //             self.cursor = Cursor::Nodes(f.exit_field());
    //         }
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(js_name = skipPendingFields)]
    // pub fn skip_pending_fields(&mut self) -> bool {
    //     let old = replace(&mut self.cursor, Cursor::Empty);
    //     match old {
    //         Cursor::Fields(f) => match f.skip_pending_fields() {
    //             EitherCursor::Nodes(n) => {
    //                 self.cursor = Cursor::Nodes(n);
    //                 false
    //             }
    //             EitherCursor::Fields(f) => {
    //                 self.cursor = Cursor::Fields(f);
    //                 true
    //             }
    //         },
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(js_name = getFieldLength)]
    // pub fn get_field_length(&self) -> i32 {
    //     match &self.cursor {
    //         Cursor::Fields(f) => f.get_field_length(),
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(js_name = firstNode)]
    // pub fn first_node(&mut self) -> bool {
    //     let old = replace(&mut self.cursor, Cursor::Empty);
    //     match old {
    //         Cursor::Fields(f) => match f.first_node() {
    //             EitherCursor::Nodes(n) => {
    //                 self.cursor = Cursor::Nodes(n);
    //                 true
    //             }
    //             EitherCursor::Fields(f) => {
    //                 self.cursor = Cursor::Fields(f);
    //                 false
    //             }
    //         },
    //         _ => panic!(),
    //     }
    // }

    // #[wasm_bindgen(js_name = enterNode)]
    // pub fn enter_node(&mut self, child_index: i32) {
    //     let old = replace(&mut self.cursor, Cursor::Empty);
    //     match old {
    //         Cursor::Fields(f) => {
    //             self.cursor = Cursor::Nodes(f.enter_node(child_index));
    //         }
    //         _ => panic!(),
    //     }
    // }
}
