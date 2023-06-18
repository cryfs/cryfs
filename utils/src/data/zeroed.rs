use super::Data;

/// A wrapper around a data object with the invariant that the data object is zeroed out.
pub struct ZeroedData<D: AsRef<[u8]> + AsMut<[u8]>> {
    data: D,
}

impl ZeroedData<Data> {
    pub fn new(len: usize) -> Self {
        Self {
            data: Data::from(vec![0; len]),
        }
    }
}

impl<D: AsRef<[u8]> + AsMut<[u8]>> ZeroedData<D> {
    pub fn fill_with_zeroes(mut data: D) -> Self {
        data.as_mut().fill(0);
        Self { data }
    }

    pub fn into_inner(self) -> D {
        self.data
    }
}
