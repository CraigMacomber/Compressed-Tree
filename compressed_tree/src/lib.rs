#[macro_use]
extern crate lazy_static;

#[cfg(target_arch = "wasm32")]
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
struct Value(Option<f64>); // TODO: more value types

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FieldKey(String);
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TreeType(String);

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
    fn field_index(&self) -> u32;

    /**
     * Index (within its parent field) of the first node in the current chunk.
     * Always less than or equal to `currentIndexInField`.
     *
     * Only valid when `mode` is `Nodes`.
     */
    fn chunk_start(&self) -> u32;

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
    fn chunk_length(&self) -> u32;

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
    fn next_node(self) -> EitherCursor<Self, Self::TFields>;

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
    fn skip_pending_fields(self) -> EitherCursor<Self::TNodes, Self>;

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
    fn get_field_length(&self) -> i32;

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

enum EitherCursor<TNodes, TFields: FieldsCursor<TNodes = TNodes>> {
    Nodes(TNodes),
    Fields(TFields),
}

pub mod basic_tree;
pub mod dummy_cursor;
pub mod forest;
pub mod wasm;
