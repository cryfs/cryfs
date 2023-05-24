use anyhow::Result;
use binary_layout::Field;
use rand::{thread_rng, RngCore};
use thiserror::Error;

use cryfs_utils::data::Data;

// TODO Make padding deterministic and check it when it's removed

binary_layout::define_layout!(padded_data, LittleEndian, {
    num_bytes: u32,
    data_and_padding: [u8],
});

#[derive(Error, Debug)]
pub enum AddPaddingError {
    #[error("Data too large. It has a length of {data_len} with a target_size of {target_size}. We should increase padding target size.")]
    DataTooLargeForTargetSize { data_len: usize, target_size: usize },

    #[error(
        "We can only pad data lengths up to u32::MAX. The data has a length of {data_len} bytes."
    )]
    DataTooLargeForU32 { data_len: usize },
}

pub fn add_padding(mut data: Data, target_size: usize) -> Result<Data, AddPaddingError> {
    let header_len = std::mem::size_of::<u32>();
    let data_len = data.len();
    let padding_len = target_size
        .checked_sub(header_len + data_len)
        .ok_or_else(|| AddPaddingError::DataTooLargeForTargetSize {
            data_len,
            target_size,
        })?;
    let data_len_u32 =
        u32::try_from(data_len).map_err(|_| AddPaddingError::DataTooLargeForU32 { data_len })?;

    data.grow_region(header_len, padding_len);

    let mut data = padded_data::View::new(data);
    data.num_bytes_mut().write(data_len_u32);
    let padding_region = &mut data.data_and_padding_mut()[data_len..];
    assert_eq!(padding_len, padding_region.len());
    thread_rng().fill_bytes(padding_region);

    Ok(data.into_storage())
}

#[derive(Error, Debug)]
pub enum RemovePaddingError {
    #[error("Padded data claims to store {data_len} bytes but the whole padded blob is only {total_len} bytes.")]
    DataTooLarge { data_len: usize, total_len: usize },

    #[error("Padded data claims to store {data_len} bytes, which is larger than usize::MAX.")]
    DataTooLargeForUsize { data_len: u32 },
}

pub fn remove_padding(mut data: Data) -> Result<Data, RemovePaddingError> {
    let data_len = padded_data::View::new(&data).num_bytes().read();
    let data_len_usize = usize::try_from(data_len)
        .map_err(|_| RemovePaddingError::DataTooLargeForUsize { data_len })?;
    if data.len() < padded_data::data_and_padding::OFFSET + data_len_usize {
        return Err(RemovePaddingError::DataTooLarge {
            data_len: data_len_usize,
            total_len: data.len(),
        });
    }
    data.shrink_to_subregion(
        padded_data::data_and_padding::OFFSET
            ..(padded_data::data_and_padding::OFFSET + data_len_usize),
    );
    Ok(data)
}

// TODO Tests
