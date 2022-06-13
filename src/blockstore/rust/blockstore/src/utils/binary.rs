use anyhow::{ensure, Context, Error, Result};
use binread::{BinRead, BinResult, ReadOptions};
use binwrite::{BinWrite, WriterOption};
use core::num::NonZeroU8;
use itertools::Itertools;
use std::collections::hash_map::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

// TODO Re-enable doc tests

/// Extension trait to deserialize an object from a stream
pub trait BinaryReadExt: Sized {
    /// Deserialize the object from the given stream and ensure that the stream
    /// is fully used. This function will return an error if the stream has more data
    /// after the object.
    fn deserialize_from_complete_stream(source: &mut (impl Read + Seek)) -> Result<Self>;

    /// Deserialize the object from the given file and ensure that the file
    /// is fully used. This function will return an error if the file has more data
    /// after the object.
    /// If the file doesn't exist, `None` is returned.
    fn deserialize_from_file(file_path: &Path) -> Result<Option<Self>>;
}

impl<T: BinRead<Args = ()> + Sized> BinaryReadExt for T {
    fn deserialize_from_complete_stream(source: &mut (impl Read + Seek)) -> Result<Self> {
        let mut read_options = ReadOptions::default();
        read_options.endian = binread::Endian::Little;
        let result = Self::read_options(source, &read_options, ())
            .map_err(|err| {
                if let binread::Error::Io(io_error) = &err {
                    if io_error.kind() == ErrorKind::UnexpectedEof {
                        Error::from(err).context("Not enough data in the stream to read the object")
                    } else {
                        err.into()
                    }
                } else {
                    err.into()
                }
            })
            .context("Tried to read object from stream")?;
        ensure_stream_is_complete(source)?;
        Ok(result)
    }

    fn deserialize_from_file(file_path: &Path) -> Result<Option<Self>> {
        match File::open(file_path) {
            Ok(file) => Ok(Some(Self::deserialize_from_complete_stream(
                &mut BufReader::new(file),
            )?)),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

fn ensure_stream_is_complete(stream: &mut (impl Read + Seek)) -> Result<()> {
    let cur_pos = stream
        .seek(SeekFrom::Current(0))
        .context("Tried to get current stream pos")?;
    let end_pos = stream
        .seek(SeekFrom::End(0))
        .context("Tried to seek to the end of the stream")?;
    let remaining_bytes = end_pos - cur_pos;
    ensure!(
        0 == remaining_bytes,
        "After successfully reading, the stream still has {} bytes left",
        remaining_bytes
    );
    Ok(())
}

/// Extension trait to serialize an object into a stream or a file.
pub trait BinaryWriteExt {
    /// Serialize the object into the given stream
    fn serialize_to_stream(&self, dest: &mut impl Write) -> Result<()>;

    /// Serialize the object into the given file.
    /// If the file already exists, it will be overwritten.
    fn serialize_to_file(&self, file_path: &Path) -> Result<()>;
}

impl<T: BinWrite> BinaryWriteExt for T {
    fn serialize_to_stream(&self, dest: &mut impl Write) -> Result<()> {
        let mut write_options = WriterOption::default();
        write_options.endian = binwrite::Endian::Little;
        self.write_options(dest, &write_options)
            .context("Tried to write object to stream")?;
        Ok(())
    }

    fn serialize_to_file(&self, file_path: &Path) -> Result<()> {
        let file = File::create(&file_path).context("Tried to create file to serialize to")?;
        self.serialize_to_stream(&mut BufWriter::new(file))?;
        Ok(())
    }
}

/// Deserialize a bool field with [binread].
///
/// # Examples
/// ```ignore
/// use binread::BinRead;
/// use cryfs_blockstore::utils::binary::read_bool;
///
/// #[derive(BinRead)]
/// struct MyStruct {
///   #[binread(parse_with = read_bool)]
///   bool_field: bool,
/// }
/// ```
///
pub fn read_bool<R: Read + Seek>(reader: &mut R, ro: &ReadOptions, _: ()) -> BinResult<bool> {
    let pos = reader.seek(SeekFrom::Current(0))?;
    let value = u8::read_options(reader, ro, ())?;
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(binread::Error::AssertFail {
            pos,
            message: format!(
                "Tried to read '{}' as a boolean value. Must be 0 or 1.",
                value
            ),
        }),
    }
}

/// Serialize a bool field with [binwrite].
///
/// # Examples
/// ```ignore
/// use binwrite::BinWrite;
/// use cryfs_blockstore::utils::binary::write_bool;
///
/// #[derive(BinWrite)]
/// struct MyStruct {
///   #[binwrite(with(write_bool))]
///   bool_field: bool,
/// }
/// ```
///
pub fn write_bool(
    v: &bool,
    writer: &mut impl Write,
    options: &WriterOption,
) -> Result<(), std::io::Error> {
    let v = if *v { 1 } else { 0 };
    u8::write_options(&v, writer, options)
}

/// Deserialize a hashmap with [binread].
///
/// # Examples
/// ```ignore
/// use binread::BinRead;
/// use std::collections::HashMap;
/// use cryfs_blockstore::utils::binary::read_hashmap;
///
/// #[derive(BinRead)]
/// struct MyStruct {
///   #[binread(parse_with = read_hashmap)]
///   some_map: HashMap<String, i64>,
/// }
/// ```
pub fn read_hashmap<K: BinRead<Args = ()> + Eq + Hash, V: BinRead<Args = ()>, R: Read + Seek>(
    reader: &mut R,
    ro: &ReadOptions,
    _: (),
) -> BinResult<HashMap<K, V>> {
    let len = u64::read_options(reader, ro, ())?;
    (0..len)
        .map(|_| {
            let key = K::read_options(reader, ro, ())?;
            let value = V::read_options(reader, ro, ())?;
            Ok((key, value))
        })
        .collect()
}

/// Serialize a hashmap with [binwrite].
///
/// # Examples
/// ```ignore
/// use binwrite::BinWrite;
/// use std::collections::HashMap;
/// use cryfs_blockstore::utils::binary::write_hashmap;
///
/// #[derive(BinWrite)]
/// struct MyStruct {
///   #[binwrite(with(write_hashmap))]
///   some_map: HashMap<String, i64>,
/// }
/// ```
pub fn write_hashmap<K: BinWrite + Eq + Hash, V: BinWrite>(
    v: &HashMap<K, V>,
    writer: &mut impl Write,
    options: &WriterOption,
) -> Result<(), std::io::Error> {
    let len = v.len() as u64;
    u64::write_options(&len, writer, options)?;
    for (key, value) in v {
        key.write_options(writer, options)?;
        value.write_options(writer, options)?;
    }
    Ok(())
}

/// Deerialize a [String] with [binwrite].
///
/// [binread] offers [NullString] which is similar, but [NullString]
/// succeeds on reading a string even if it is terminated by EOF instead
/// of NULL. This function is more strict and always expects a NULL in the end.
///
/// # Examples
/// ```ignore
/// use binread::BinRead;
/// use cryfs_blockstore::utils::binary::read_null_string;
///
/// #[derive(BinRead)]
/// struct MyStruct {
///   #[binread(parse_with = read_null_string)]
///   some_str: String,
/// }
/// ```
pub fn read_null_string<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    _: (),
) -> BinResult<Vec<NonZeroU8>> {
    let pos = reader.seek(SeekFrom::Current(0))?;
    let mut reader = reader.bytes().peekable();
    let data: BinResult<Vec<NonZeroU8>> = reader
        .by_ref()
        .peeking_take_while(|x| !matches!(x, Ok(0)))
        .map(|x| Ok(x.map(|byte| unsafe { NonZeroU8::new_unchecked(byte) })?))
        .collect();
    let data = data?;

    if reader.next().transpose()? == Some(b'\0') {
        Ok(data)
    } else {
        Err(binread::Error::AssertFail {
            pos,
            message: String::from(
                "Expected string to be terminated by a nullbyte but found EOF instead.",
            ),
        })
    }
}

/// Serialize a [String] with [binwrite].
///
/// [NullString] is a class provided by [binread] but
/// [binwrite] doesn't natively offer a way to serialize it.
/// You can use this helper to serialize it.
///
/// # Examples
/// ```ignore
/// use binwrite::BinWrite;
/// use cryfs_blockstore::utils::binary::write_null_string;
///
/// #[derive(BinWrite)]
/// struct MyStruct {
///   #[binwrite(with(write_null_string))]
///   some_str: String,
/// }
/// ```
pub fn write_null_string(
    str: &Vec<NonZeroU8>,
    writer: &mut impl Write,
    options: &WriterOption,
) -> Result<(), std::io::Error> {
    for c in str {
        c.get().write_options(writer, options)?;
    }
    // and add null byte
    u8::write_options(&0, writer, options)
}

#[cfg(test)]
pub mod testutils {
    use super::*;
    use std::fmt::Debug;
    use std::io::Cursor;

    /// Take some parts made of binary data and concatenate
    /// them together into one binary vector.
    pub fn binary(parts: &[&[u8]]) -> Vec<u8> {
        let mut data = Vec::new();
        for part in parts {
            Write::write(&mut data, part).unwrap();
        }
        data
    }

    /// Deserialize an object from binary data
    pub fn deserialize<T: BinRead<Args = ()>>(serialized: &[u8]) -> Result<T> {
        let mut cursor = Cursor::new(serialized);
        T::deserialize_from_complete_stream(&mut cursor)
    }

    /// Test that
    /// * serializing `object` yields one of the given `serialized_variants`
    /// * deserializing each of the `serialized_variants` yields `object`
    pub fn test_serialize_deserialize<T>(object: T, serialized_variants: &[&[u8]])
    where
        T: BinRead<Args = ()> + BinWrite + PartialEq + Debug,
    {
        for serialized in serialized_variants {
            let loaded = deserialize(serialized).unwrap();
            assert_eq!(
                object, loaded,
                "Deserialization didn't match expected output"
            );
        }

        let mut saved = Vec::new();
        object.serialize_to_stream(&mut saved).unwrap();
        assert!(
            serialized_variants.contains(&&*saved),
            "Serialization didn't match expected output"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::testutils::*;
    use super::*;

    #[derive(BinRead, BinWrite, PartialEq, Debug)]
    struct MyStruct {
        field1: u32,
        field2: i8,
    }

    impl Default for MyStruct {
        fn default() -> Self {
            Self {
                field1: 100,
                field2: -100,
            }
        }
    }

    mod deserialize_from_complete_stream {
        use super::*;

        #[test]
        fn success() {
            test_serialize_deserialize(
                MyStruct {
                    field1: 50_000,
                    field2: -20,
                },
                &[&binary(&[&50_000u32.to_le_bytes(), &(-20i8).to_le_bytes()])],
            );
        }

        #[test]
        fn error_too_little_data() {
            let error = deserialize::<MyStruct>(&binary(&[&50_000u32.to_le_bytes()])).unwrap_err();
            let error_msg = format!("{:?}", error);

            assert!(
                error_msg.contains("Not enough data in the stream to read the object"),
                "Wrong error message: {:?}",
                error_msg
            );
        }

        #[test]
        fn error_too_much_data() {
            let error = deserialize::<MyStruct>(&binary(&[
                &50_000u32.to_le_bytes(),
                &(-20i8).to_le_bytes(),
                b"1234567",
            ]))
            .unwrap_err();
            let error_msg = format!("{:?}", error);

            assert!(
                error_msg.contains("After successfully reading, the stream still has 7 bytes left"),
                "Wrong error message: {:?}",
                error_msg
            );
        }
    }

    mod deserialize_from_file {
        use super::*;
        use tempdir::TempDir;

        #[test]
        fn nonexisting_file() {
            let tempdir = TempDir::new("").unwrap();
            let file_path = tempdir.path().join("file");
            assert_eq!(None, MyStruct::deserialize_from_file(&file_path).unwrap());
        }

        #[test]
        fn existing_file() {
            let tempdir = TempDir::new("").unwrap();
            let file_path = tempdir.path().join("file");
            let object = MyStruct {
                field1: 50_000,
                field2: 10,
            };
            object.serialize_to_file(&file_path).unwrap();
            let loaded = MyStruct::deserialize_from_file(&file_path).unwrap();
            assert_eq!(Some(object), loaded);
        }
    }

    mod read_write_bool {
        use super::*;

        #[derive(BinRead, BinWrite, Debug, PartialEq)]
        struct MyStruct {
            #[binread(parse_with = read_bool)]
            #[binwrite(with(write_bool))]
            field: bool,
        }

        #[test]
        fn success_true() {
            test_serialize_deserialize(MyStruct { field: true }, &[&binary(&[&1u8.to_le_bytes()])]);
        }

        #[test]
        fn success_false() {
            test_serialize_deserialize(
                MyStruct { field: false },
                &[&binary(&[&0u8.to_le_bytes()])],
            );
        }

        #[test]
        fn error_invalid_value() {
            let error = deserialize::<MyStruct>(&binary(&[&2u8.to_le_bytes()])).unwrap_err();
            let error_msg = format!("{:?}", error);

            assert!(
                error_msg.contains("Tried to read '2' as a boolean value. Must be 0 or 1."),
                "Wrong error message: {:?}",
                error_msg
            );
        }
    }

    mod read_write_hashmap {
        use super::*;
        use common_macros::hash_map;

        #[derive(BinRead, BinWrite, Debug, PartialEq)]
        struct MyStruct {
            #[binread(parse_with = read_hashmap)]
            #[binwrite(with(write_hashmap))]
            field: HashMap<u32, u64>,
        }

        #[test]
        fn success_empty() {
            test_serialize_deserialize(
                MyStruct {
                    field: HashMap::new(),
                },
                &[&binary(&[&0u64.to_le_bytes()])],
            );
        }

        #[test]
        fn success_nonempty() {
            let first_entry_serialized = binary(&[&2u32.to_le_bytes(), &5u64.to_le_bytes()]);
            let second_entry_serialized =
                binary(&[&100u32.to_le_bytes(), &10_000u64.to_le_bytes()]);
            test_serialize_deserialize(
                MyStruct {
                    field: hash_map! {
                        2 => 5,
                        100 => 10_000,
                    },
                },
                &[
                    &binary(&[
                        &2u64.to_le_bytes(),
                        &first_entry_serialized,
                        &second_entry_serialized,
                    ]),
                    &binary(&[
                        &2u64.to_le_bytes(),
                        &second_entry_serialized,
                        &first_entry_serialized,
                    ]),
                ],
            );
        }

        #[test]
        fn error_too_short_for_length_field() {
            let error = deserialize::<MyStruct>(&binary(&[&2u8.to_le_bytes()])).unwrap_err();
            let error_msg = format!("{:?}", error);

            assert!(
                error_msg.contains("Not enough data in the stream to read the object"),
                "Wrong error message: {:?}",
                error_msg
            );
        }

        #[test]
        fn error_too_short_for_num_entries() {
            let error = deserialize::<MyStruct>(&binary(&[
                &2u64.to_le_bytes(),
                &100u32.to_le_bytes(),
                &10_000u64.to_le_bytes(),
            ]))
            .unwrap_err();
            let error_msg = format!("{:?}", error);

            assert!(
                error_msg.contains("Not enough data in the stream to read the object"),
                "Wrong error message: {:?}",
                error_msg
            );
        }
    }

    mod write_null_string {
        use super::*;

        #[derive(BinRead, BinWrite, Debug, PartialEq)]
        struct MyStruct {
            #[binwrite(with(write_null_string))]
            #[binread(parse_with = read_null_string)]
            field: Vec<NonZeroU8>,
            #[binwrite(with(write_null_string))]
            #[binread(parse_with = read_null_string)]
            field2: Vec<NonZeroU8>,
        }

        fn make_null_string(s: &[u8]) -> Vec<NonZeroU8> {
            s.iter()
                .map(|c| NonZeroU8::new(*c).unwrap())
                .collect::<Vec<_>>()
                .into()
        }

        #[test]
        fn success() {
            test_serialize_deserialize(
                MyStruct {
                    field: make_null_string(b"Hello "),
                    field2: make_null_string(b"World"),
                },
                &[&binary(&[b"Hello \0World\0"])],
            );
        }

        #[test]
        fn failure_missing_nullbyte() {
            let error = deserialize::<MyStruct>(&binary(&[b"Hello \0World"])).unwrap_err();
            let error_msg = format!("{:?}", error);

            assert!(
                error_msg.contains(
                    "Expected string to be terminated by a nullbyte but found EOF instead."
                ),
                "Wrong error message: {:?}",
                error_msg
            );
        }

        #[test]
        fn failure_empty_data() {
            let error = deserialize::<MyStruct>(&[]).unwrap_err();
            let error_msg = format!("{:?}", error);

            assert!(
                error_msg.contains(
                    "Expected string to be terminated by a nullbyte but found EOF instead."
                ),
                "Wrong error message: {:?}",
                error_msg
            );
        }
    }
}
