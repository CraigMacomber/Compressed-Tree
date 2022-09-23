pub type ImSlice<'a> = &'a [u8]; //im_rc::vector::Focus<'a, u8>;

pub fn slice_with_length(
    focus: ImSlice<'_>,
    offset: usize,
    length: usize,
) -> ImSlice<'_> {
    &focus[offset..offset + length]
}
