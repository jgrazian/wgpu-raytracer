pub trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
    fn bytes_size(&self) -> usize;
}
