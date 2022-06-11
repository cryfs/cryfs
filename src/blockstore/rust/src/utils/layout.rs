use std::marker::PhantomData;

pub struct Field<T> {
    offset: usize,
    _phantom: PhantomData<T>,
}

impl<T> Field<T> {
    pub const fn at_offset(offset: usize) -> Self {
        Self {
            offset,
            _phantom: PhantomData {},
        }
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }
}

macro_rules! int_field {
    ($type:ident) => {
        impl Field<$type> {
            pub fn read(&self, storage: &[u8]) -> $type {
                let mut value = [0; std::mem::size_of::<$type>()];
                value.copy_from_slice(
                    &storage[self.offset..(self.offset + std::mem::size_of::<$type>())],
                );
                $type::from_le_bytes(value)
            }
            pub fn write(&self, storage: &mut [u8], value: $type) {
                storage[self.offset..(self.offset + std::mem::size_of::<$type>())]
                    .copy_from_slice(&value.to_le_bytes());
            }
            pub const fn size() -> usize {
                std::mem::size_of::<$type>()
            }
        }
    };
}

int_field!(i8);
int_field!(i16);
int_field!(i32);
int_field!(i64);
int_field!(u8);
int_field!(u16);
int_field!(u32);
int_field!(u64);

impl Field<&[u8]> {
    pub fn data<'a>(&self, storage: &'a [u8]) -> &'a [u8] {
        &storage[self.offset..]
    }

    pub fn data_mut<'a>(&self, storage: &'a mut [u8]) -> &'a mut [u8] {
        &mut storage[self.offset..]
    }
}

#[macro_export]
macro_rules! define_layout_impl {
    ($offset_accumulator: expr, {}) => {};
    ($offset_accumulator: expr, {$name: ident : $type: ty $(, $name_tail: ident : $type_tail: ty)*}) => {
        pub const $name: $crate::util::layout::Field<$type> = $crate::util::layout::Field::at_offset($offset_accumulator);
        define_layout_impl!($offset_accumulator + $crate::util::layout::Field::<$type>::size(), {$($name_tail : $type_tail),*});
    };
}

#[macro_export]
macro_rules! define_layout {
    ($name: ident, {$($field_name: ident : $field_type: ty),* $(,)?}) => {
        mod $name {
            use $crate::define_layout_impl;
            define_layout_impl!(0, {$($field_name : $field_type),*});
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::{rngs::StdRng, RngCore, SeedableRng};

    fn data_region(size: usize, seed: u64) -> Vec<u8> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut res = vec![0; size];
        rng.fill_bytes(&mut res);
        res
    }

    #[test]
    fn test_i8() {
        const FIELD1: Field<i8> = Field::at_offset(5);
        const FIELD2: Field<i8> = Field::at_offset(20);
        let mut storage = vec![0; 1024];
        FIELD1.write(&mut storage, 50);
        FIELD2.write(&mut storage, -20);
        assert_eq!(50, FIELD1.read(&storage));
        assert_eq!(-20, FIELD2.read(&storage));
        assert_eq!(1, Field::<i8>::size());
    }

    #[test]
    fn test_i16() {
        const FIELD1: Field<i16> = Field::at_offset(5);
        const FIELD2: Field<i16> = Field::at_offset(20);
        let mut storage = vec![0; 1024];
        FIELD1.write(&mut storage, 500);
        FIELD2.write(&mut storage, -2000);
        assert_eq!(500, FIELD1.read(&storage));
        assert_eq!(-2000, FIELD2.read(&storage));
        assert_eq!(2, Field::<i16>::size());
    }

    #[test]
    fn test_i32() {
        const FIELD1: Field<i32> = Field::at_offset(5);
        const FIELD2: Field<i32> = Field::at_offset(20);
        let mut storage = vec![0; 1024];
        FIELD1.write(&mut storage, (2i32.pow(29)));
        FIELD2.write(&mut storage, -(2i32.pow(25)));
        assert_eq!(2i32.pow(29), FIELD1.read(&storage));
        assert_eq!(-2i32.pow(25), FIELD2.read(&storage));
        assert_eq!(4, Field::<i32>::size());
    }

    #[test]
    fn test_i64() {
        const FIELD1: Field<i64> = Field::at_offset(5);
        const FIELD2: Field<i64> = Field::at_offset(20);
        let mut storage = vec![0; 1024];
        FIELD1.write(&mut storage, (2i64.pow(40)));
        FIELD2.write(&mut storage, -(2i64.pow(50)));
        assert_eq!(2i64.pow(40), FIELD1.read(&storage));
        assert_eq!(-2i64.pow(50), FIELD2.read(&storage));
        assert_eq!(8, Field::<i64>::size());
    }
    #[test]
    fn test_u8() {
        const FIELD1: Field<u8> = Field::at_offset(5);
        const FIELD2: Field<u8> = Field::at_offset(20);
        let mut storage = vec![0; 1024];
        FIELD1.write(&mut storage, 50);
        FIELD2.write(&mut storage, 20);
        assert_eq!(50, FIELD1.read(&storage));
        assert_eq!(20, FIELD2.read(&storage));
        assert_eq!(1, Field::<u8>::size());
    }

    #[test]
    fn test_u16() {
        const FIELD1: Field<u16> = Field::at_offset(5);
        const FIELD2: Field<u16> = Field::at_offset(20);
        let mut storage = vec![0; 1024];
        FIELD1.write(&mut storage, 500);
        FIELD2.write(&mut storage, 2000);
        assert_eq!(500, FIELD1.read(&storage));
        assert_eq!(2000, FIELD2.read(&storage));
        assert_eq!(2, Field::<u16>::size());
    }

    #[test]
    fn test_u32() {
        const FIELD1: Field<u32> = Field::at_offset(5);
        const FIELD2: Field<u32> = Field::at_offset(20);
        let mut storage = vec![0; 1024];
        FIELD1.write(&mut storage, (2u32.pow(29)));
        FIELD2.write(&mut storage, (2u32.pow(25)));
        assert_eq!(2u32.pow(29), FIELD1.read(&storage));
        assert_eq!(2u32.pow(25), FIELD2.read(&storage));
        assert_eq!(4, Field::<u32>::size());
    }

    #[test]
    fn test_u64() {
        const FIELD1: Field<u64> = Field::at_offset(5);
        const FIELD2: Field<u64> = Field::at_offset(20);
        let mut storage = vec![0; 1024];
        FIELD1.write(&mut storage, (2u64.pow(40)));
        FIELD2.write(&mut storage, (2u64.pow(50)));
        assert_eq!(2u64.pow(40), FIELD1.read(&storage));
        assert_eq!(2u64.pow(50), FIELD2.read(&storage));
        assert_eq!(8, Field::<u64>::size());
    }

    #[test]
    fn test_slice() {
        const FIELD1: Field<&[u8]> = Field::at_offset(5);
        const FIELD2: Field<&[u8]> = Field::at_offset(7);
        let mut storage = vec![0; 1024];
        FIELD1.data_mut(&mut storage)[..5].copy_from_slice(&[10, 20, 30, 40, 50]);
        FIELD2.data_mut(&mut storage)[..5].copy_from_slice(&[60, 70, 80, 90, 100]);
        assert_eq!(&[10, 20, 60, 70, 80], &FIELD1.data(&storage)[..5]);
        assert_eq!(&[60, 70, 80, 90, 100], &FIELD2.data(&storage)[..5]);
    }

    #[test]
    fn test_layout_empty() {
        define_layout!(empty, {});
    }

    #[test]
    fn test_layout_sliceonly() {
        define_layout!(sliceonly, { data: &[u8] });

        let storage = data_region(1024, 5);
        assert_eq!(&storage, sliceonly::data.data(&storage));
    }

    #[test]
    fn test_layout_noslice() {
        define_layout!(noslice, {
            first: i8,
            second: i64,
            third: u16,
        });

        let mut storage = data_region(1024, 5);

        assert_eq!(0, noslice::first.offset());
        assert_eq!(1, noslice::second.offset());
        assert_eq!(9, noslice::third.offset());

        noslice::first.write(&mut storage, 60);
        noslice::second.write(&mut storage, -100_000_000_000);
        noslice::third.write(&mut storage, 1_000);

        assert_eq!(60, noslice::first.read(&storage));
        assert_eq!(-100_000_000_000, noslice::second.read(&storage));
        assert_eq!(1_000, noslice::third.read(&storage));
    }

    #[test]
    fn test_layout_withslice() {
        define_layout!(withslice, {
            first: i8,
            second: i64,
            third: u16,
            fourth: &[u8],
        });

        let mut storage = data_region(1024, 5);

        assert_eq!(0, withslice::first.offset());
        assert_eq!(1, withslice::second.offset());
        assert_eq!(9, withslice::third.offset());
        assert_eq!(11, withslice::fourth.offset());
        assert_eq!(1024 - 11, withslice::fourth.data(&storage).len());
        assert_eq!(1024 - 11, withslice::fourth.data_mut(&mut storage).len());

        withslice::first.write(&mut storage, 60);
        withslice::second.write(&mut storage, -100_000_000_000);
        withslice::third.write(&mut storage, 1_000);
        withslice::fourth
            .data_mut(&mut storage)
            .copy_from_slice(&data_region(1024 - 11, 6));

        assert_eq!(60, withslice::first.read(&storage));
        assert_eq!(-100_000_000_000, withslice::second.read(&storage));
        assert_eq!(1_000, withslice::third.read(&storage));
        assert_eq!(&data_region(1024 - 11, 6), withslice::fourth.data(&storage));
    }

    #[test]
    fn can_be_created_with_and_without_trailing_comma() {
        define_layout!(first, { field: u8 });
        define_layout!(second, {
            field: u8,
            second: u16
        });
        define_layout!(third, {
            field: u8,
        });
        define_layout!(fourth, {
            field: u8,
            second: u16,
        });
    }
}
