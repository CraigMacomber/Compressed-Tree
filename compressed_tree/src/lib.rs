// #![no_std]

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn greet(name: &str) {
    console::log_1(&format!("Hello, {}!", name).into());
}



#[cfg(test)]
mod tests {
    #[test]
    fn basic_test() {
        assert_eq!(0, 0);
    }
}

trait UpPath {}
struct Value(Option<JsValue>);
struct FieldKey;
struct TreeType;

trait NodesCursor: Sized {
    type TFields: FieldsCursor<TNodes = Self>;
    // ********** APIs for when mode = Nodes ********** //

    /**
     * @returns a path to the current node.
     *
     * Only valid when `mode` is `Nodes`.
     * Assumes this cursor has a root node where its field keys are actually detached sequences.
     * If the cursor is not rooted at such a node,
     * calling this function is invalid, and the returned UpPath (if any) may not be meaningful.
     * This requirement exists because {@link UpPath}s are absolute paths
     * and thus must be rooted in a detached sequence.
     * TODO: consider adding an optional base path to append to remove/clarify this restriction.
     */
    // fn getPath(&self) -> impl UpPath; //  | undefined

    /**
     * Index (within its parent field) of the current node.
     *
     * Only valid when `mode` is `Nodes`.
     */
    fn field_index(&self) -> i32;

    /**
     * Index (within its parent field) of the first node in the current chunk.
     * Always less than or equal to `currentIndexInField`.
     *
     * Only valid when `mode` is `Nodes`.
     */
    fn chunk_start(&self) -> i32;

    /**
     * Length of current chunk.
     * Since an entire chunk always has the same `pending` value,
     * can be used to help skip over all of a pending chunk at once.
     *
     * TODO:
     * Add optional APIs to access underlying chunks so readers can
     * accelerate processing of chunk formats they understand.
     *
     * Only valid when `mode` is `Nodes`.
     */
    fn chunk_length(&self) -> i32;

    /**
     * Moves `offset` nodes in the field.
     * If seeking to exactly past either end,
     * returns false and navigates up to the parent field (setting mode to `Fields`).
     *
     * Allowed if mode is `Nodes`.
     */
    fn seek_nodes(self, offset: i32) -> EitherCursor<Self, Self::TFields>;

    /**
     * The same as `seek_nodes(1)`, but might be faster.
     */
    fn next_node(&self) -> EitherCursor<Self, Self::TFields>;

    /**
     * Navigate up to parent field.
     * Sets mode to `Fields`
     *
     * Same as seek i32.POSITIVE_INFINITY, but only valid when `mode` is `Nodes`.
     *
     * TODO: what to do if at root?
     * TODO: Maybe merge with upToNode to make a single "Up"?
     */
    fn exit_node(self) -> Self::TFields;

    // ********** APIs for when mode = Nodes and not pending ********** //

    /**
     * Enters the first field (setting mode to `Fields`)
     * so fields can be iterated with `nextField` and `skipPendingFields`.
     *
     * If there are no fields, mode is returned to `Nodes` and false is returned.
     *
     * Allowed when `mode` is `Nodes` and not `pending`.
     */
    fn first_field(self) -> EitherCursor<Self, Self::TFields>;

    /**
     * Navigate to the field with the specified `key` and set the mode to `Fields`.
     *
     * Only valid when `mode` is `Nodes`, and not `pending`.
     */
    fn enter_field(self, key: FieldKey) -> EitherCursor<Self, Self::TFields>;

    /**
     * The type of the currently selected node.
     *
     * Only valid when `mode` is `Nodes`, and not `pending`.
     */
    fn node_type(&self) -> TreeType;

    /**
     * The value associated with the currently selected node.
     *
     * Only valid when `mode` is `Nodes`, and not `pending`.
     */
    fn value(&self) -> Value;
}

trait FieldsCursor: Sized {
    type TNodes: NodesCursor<TFields = Self>;
    // ********** APIs for when mode = Fields ********** //

    /**
     * Moves the "current field" forward one in an arbitrary field traversal order.
     *
     * If there is no remaining field to iterate to,
     * returns false and navigates up to the parent setting the mode to `Nodes`.
     *
     * Order of fields is only guaranteed to be consistent thorough a single iteration.
     *
     * If skipPending, skip past fields which are currently pending.
     * This can be used to skip to the end of a large i32 of consecutive pending fields.
     *
     * Allowed when `mode` is `Fields`.
     */
    fn next_field(self) -> EitherCursor<Self::TNodes, Self>;

    /**
     * Navigate up to parent node.
     * Sets mode to `Nodes`
     *
     * Only valid when `mode` is `Fields`.
     *
     * TODO: what to do if at root?
     */
    fn exit_field(self) -> Self::TNodes;

    /**
     * Moves the "current field" forward until `pending` is `false`.
     *
     * If there are no remaining field to iterate to,
     * returns false and navigates up to the parent setting the mode to `Nodes`.
     *
     * Order of fields is only guaranteed to be consistent thorough a single iteration.
     *
     * Allowed when `mode` is `Fields`.
     */
    fn skip_pending_fields() -> EitherCursor<Self::TNodes, Self>;

    // ********** APIs for when mode = Fields, and not pending ********** //

    /**
     * Returns the FieldKey for the current field.
     *
     * Allowed when `mode` is `Fields`, and not `pending`.
     */
    // fn getFieldKey() -> FieldKey;

    /**
     * @returns the i32 of immediate children in the current field.
     *
     * Allowed when `mode` is `Fields`, and not `pending`.
     */
    fn get_field_length() -> i32;

    /**
     * Moves to the first node of the selected field, setting mode to `Nodes`.
     *
     * If field is empty, returns false instead.
     *
     * Allowed when `mode` is `Fields`, and not `pending`.
     */
    fn first_node(self) -> EitherCursor<Self::TNodes, Self>;

    /**
     * Sets current node to the node at the provided `index` of the current field.
     *
     * Allowed when `mode` is `Fields`, and not `pending`.
     * Sets mode to `Nodes`.
     */
    fn enter_node(self, child_index: i32) -> Self::TNodes;
}

/**
 * A stateful low-level interface for reading tree data.
 */
trait TreeCursor: FieldsCursor + NodesCursor {
    /**
     * What kind of place the cursor is at.
     * Determines which operations are allowed.
     */
    fn mode(&self) -> CursorLocationType;

    /*
     * True iff the current field or node (depending on mode) is "pending",
     * meaning that it has not been downloaded.
     */
    fn pending(&self) -> bool;
}

enum CursorLocationType {
    /**
     * Can iterate through nodes in a field.
     * At a "current node".
     */
    Nodes,

    /**
     * Can iterate through fields of a node.
     * At a "current field".
     */
    Fields,
}

enum EitherCursor<TNodes, TFields: FieldsCursor<TNodes = TNodes>> {
    Nodes(TNodes),
    Fields(TFields),
}

impl<TNodes: NodesCursor<TFields = TFields>, TFields: FieldsCursor<TNodes = TNodes>> FieldsCursor
    for EitherCursor<TNodes, TFields>
{
    type TNodes = Self;

    fn next_field(self) -> EitherCursor<Self::TNodes, Self> {
        match self {
            EitherCursor::Fields(fields) => {
                match fields.next_field() {
                    EitherCursor::Nodes(n) => EitherCursor::Nodes(Self::Nodes(n)),
                    EitherCursor::Fields(f) => EitherCursor::Fields(Self::Fields(f)),
                }
            }
            EitherCursor::Nodes(_) => panic!(),
        }
    }

    fn exit_field(self) -> Self::TNodes {
        match self {
            EitherCursor::Fields(fields) => Self::Nodes(fields.exit_field()),
            EitherCursor::Nodes(_) => panic!(),
        }
    }

    fn skip_pending_fields() -> EitherCursor<Self::TNodes, Self> {
        todo!()
    }

    fn get_field_length() -> i32 {
        todo!()
    }

    fn first_node(self) -> EitherCursor<Self::TNodes, Self> {
        todo!()
    }

    fn enter_node(self, child_index: i32) -> Self::TNodes {
        todo!()
    }
}

impl<TNodes: NodesCursor<TFields = TFields>, TFields: FieldsCursor<TNodes = TNodes>> NodesCursor for EitherCursor<TNodes, TFields> {
    type TFields = Self;

    fn field_index(&self) -> i32 {
        match self {
            EitherCursor::Nodes(_) => self.field_index(),
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn chunk_start(&self) -> i32 {
        match self {
            EitherCursor::Nodes(_) => self.chunk_start(),
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn chunk_length(&self) -> i32 {
        match self {
            EitherCursor::Nodes(_) => self.chunk_length(),
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn seek_nodes(self, offset: i32) -> EitherCursor<Self, Self::TFields> {
        match self {
            EitherCursor::Nodes(_) => self.seek_nodes(offset),
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn next_node(&self) -> EitherCursor<Self, Self::TFields> {
        match self {
            EitherCursor::Nodes(_) => self.next_node(),
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn exit_node(self) -> Self::TFields {
        match self {
            EitherCursor::Nodes(_) => self.exit_node(),
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn value(&self) -> Value {
        match self {
            EitherCursor::Nodes(_) => self.value(),
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn first_field(self) -> EitherCursor<Self, Self::TFields> {
        match self {
            EitherCursor::Nodes(n) => {
                match n.first_field() {
                    EitherCursor::Nodes(n) => EitherCursor::Nodes(Self::Nodes(n)),
                    EitherCursor::Fields(f) => EitherCursor::Fields(Self::Fields(f)),
                }
            },
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn enter_field(self, key: FieldKey) -> EitherCursor<Self, Self::TFields> {
        match self {
            EitherCursor::Nodes(_) => self.enter_field(key),
            EitherCursor::Fields(_) => panic!(),
        }
    }

    fn node_type(&self) -> TreeType {
        match self {
            EitherCursor::Nodes(_) => self.node_type(),
            EitherCursor::Fields(_) => panic!(),
        }
    }
}

impl<TNodes: NodesCursor<TFields = TFields>, TFields: FieldsCursor<TNodes = TNodes>> TreeCursor for EitherCursor<TNodes, TFields> {
    fn mode(&self) -> CursorLocationType {
        match self {
            EitherCursor::Nodes(_) => CursorLocationType::Nodes,
            EitherCursor::Fields(_) => CursorLocationType::Fields,
        }
    }

    fn pending(&self) -> bool {
        // For now only support non-pending data.
        false
    }
}


struct BasicNodes {}

impl NodesCursor for BasicNodes {
    type TFields = BasicFields;

    fn field_index(&self) -> i32 {
        todo!()
    }

    fn chunk_start(&self) -> i32 {
        todo!()
    }

    fn chunk_length(&self) -> i32 {
        todo!()
    }

    fn seek_nodes(self, offset: i32) -> EitherCursor<Self, Self::TFields> {
        todo!()
    }

    fn next_node(&self) -> EitherCursor<Self, Self::TFields> {
        todo!()
    }

    fn exit_node(self) -> Self::TFields {
        todo!()
    }


    fn value(&self) -> Value {
        todo!()
    }

    fn first_field(self) -> EitherCursor<Self, Self::TFields> {
        todo!()
    }

    fn enter_field(self, key: FieldKey) -> EitherCursor<Self, Self::TFields> {
        todo!()
    }

    fn node_type(&self) -> TreeType {
        todo!()
    }
} 

struct BasicFields {}

impl FieldsCursor for BasicFields {
    type TNodes = BasicNodes;

    fn next_field(self) -> EitherCursor<Self::TNodes, Self> {
        todo!()
    }

    fn exit_field(self) -> Self::TNodes {
        todo!()
    }

    fn skip_pending_fields() -> EitherCursor<Self::TNodes, Self>  {
        todo!()
    }

    fn get_field_length() -> i32 {
        todo!()
    }

    fn first_node(self) -> EitherCursor<Self::TNodes, Self> {
        todo!()
    }

    fn enter_node(self, child_index: i32) -> Self::TNodes {
        todo!()
    }
} 

#[wasm_bindgen]
pub struct WasmCursor(EitherCursor<BasicNodes, BasicFields>);

#[wasm_bindgen]
impl WasmCursor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        todo!()
    }

    #[wasm_bindgen(getter)]
    pub fn mode(&self) -> i32 {
        match self.0 {
            EitherCursor::Nodes(_) => 0,
            EitherCursor::Fields(_) => 1,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn pending(&self) -> bool {
        self.0.pending()
    }
}