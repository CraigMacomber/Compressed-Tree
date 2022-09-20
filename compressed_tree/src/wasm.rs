use std::mem::replace;

use wasm_bindgen::prelude::*;

use crate::{
    basic_tree::{BasicFields, BasicNodes},
    EitherCursor, FieldsCursor, NodesCursor, FieldKey,
};

#[wasm_bindgen]
pub struct WasmCursor(Cursor);

enum Cursor {
    Nodes(BasicNodes),
    Fields(BasicFields),
    Empty,
}

#[wasm_bindgen]
impl WasmCursor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        todo!()
    }

    #[wasm_bindgen(getter)]
    pub fn mode(&self) -> i32 {
        match self.0 {
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
    pub fn field_index(&self) -> i32 {
        match &self.0 {
            Cursor::Nodes(n) => n.field_index(),
            _ => panic!(),
        }
    }

    #[wasm_bindgen(getter, js_name = chunkStart)]
    pub fn chunk_start(&self) -> i32 {
        match &self.0 {
            Cursor::Nodes(n) => n.chunk_start(),
            _ => panic!(),
        }
    }

    #[wasm_bindgen(getter, js_name = chunkLength)]
    pub fn chunk_length(&self) -> i32 {
        match &self.0 {
            Cursor::Nodes(n) => n.chunk_length(),
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = seekNodes)]
    pub fn seek_nodes(&mut self, offset: i32) -> bool {
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => match n.seek_nodes(offset) {
                EitherCursor::Nodes(n) => {
                    self.0 = Cursor::Nodes(n);
                    true
                }
                EitherCursor::Fields(f) => {
                    self.0 = Cursor::Fields(f);
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
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => {
                self.0 = Cursor::Fields(n.exit_node());
            }
            _ => panic!(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> Option<f64> {
        match &self.0 {
            Cursor::Nodes(n) => n.value().0,
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = firstField)]
    pub fn first_field(&mut self) -> bool {
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => match n.first_field() {
                EitherCursor::Nodes(n) => {
                    self.0 = Cursor::Nodes(n);
                    false
                }
                EitherCursor::Fields(f) => {
                    self.0 = Cursor::Fields(f);
                    true
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = enterField)]
    pub fn enter_field(&mut self, key: String) -> bool {
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Nodes(n) => {
                match n.enter_field(FieldKey(key)) {
                    EitherCursor::Nodes(n) => {
                        self.0 = Cursor::Nodes(n);
                        false
                    },
                    EitherCursor::Fields(f) => {
                        self.0 = Cursor::Fields(f);
                        true
                    },
                }
            }
            _ => panic!(),
        }
    }

    #[wasm_bindgen(getter, js_name = type)]
    pub fn node_type(&self) -> String {
        match &self.0 {
            Cursor::Nodes(n) => n.node_type().0,
            _ => panic!(),
        }
    }

    ///////////////////////////

    #[wasm_bindgen(js_name = nextField)]
    pub fn next_field(&mut self) -> bool {
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Fields(f) => match f.next_field() {
                EitherCursor::Nodes(n) => {
                    self.0 = Cursor::Nodes(n);
                    false
                }
                EitherCursor::Fields(f) => {
                    self.0 = Cursor::Fields(f);
                    true
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = exitField)]
    pub fn exit_field(&mut self) {
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Fields(f) => {
                self.0 = Cursor::Nodes(f.exit_field());
            }
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = skipPendingFields)]
    pub fn skip_pending_fields(&mut self) -> bool {
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Fields(f) => match f.skip_pending_fields() {
                EitherCursor::Nodes(n) => {
                    self.0 = Cursor::Nodes(n);
                    false
                }
                EitherCursor::Fields(f) => {
                    self.0 = Cursor::Fields(f);
                    true
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = getFieldLength)]
    pub fn get_field_length(&self) -> i32 {
        match &self.0 {
            Cursor::Fields(f) => f.get_field_length(),
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = firstNode)]
    pub fn first_node(&mut self) -> bool {
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Fields(f) => match f.first_node() {
                EitherCursor::Nodes(n) => {
                    self.0 = Cursor::Nodes(n);
                    true
                }
                EitherCursor::Fields(f) => {
                    self.0 = Cursor::Fields(f);
                    false
                }
            },
            _ => panic!(),
        }
    }

    #[wasm_bindgen(js_name = enterNode)]
    pub fn enter_node(&mut self, child_index: i32) {
        let old = replace(&mut self.0, Cursor::Empty);
        match old {
            Cursor::Fields(f) => {
                self.0 = Cursor::Nodes(f.enter_node(child_index));
            }
            _ => panic!(),
        }
    }
}
