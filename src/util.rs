pub(crate) fn u32_from_usize(x: usize) -> u32 {
    u32::try_from(x).expect("u32_from_usize() failed")
}
