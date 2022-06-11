use binread::BinRead;
use binwrite::BinWrite;
use core::num::NonZeroU8;
use std::collections::hash_map::HashMap;
use std::hash::Hash;

use crate::blockstore::BlockId;
use crate::utils::binary::{
    read_bool, read_hashmap, read_null_string, write_bool, write_hashmap, write_null_string,
};

#[derive(PartialEq, Eq, Debug, Hash, BinRead, BinWrite, Clone, Copy)]
pub struct ClientId {
    // TODO Tuple struct would be better but https://github.com/jam1garner/binwrite/issues/3
    pub(super) id: u32,
}

#[derive(PartialEq, Eq, Debug, Hash, PartialOrd, BinRead, BinWrite, Clone, Copy)]
pub struct BlockVersion {
    // TODO Tuple struct would be better but https://github.com/jam1garner/binwrite/issues/3
    pub(super) version: u64,
}

impl BlockVersion {
    pub fn increment(&mut self) {
        self.version += 1;
    }
}

const FORMAT_VERSION_HEADER: &[u8] = b"cryfs.integritydata.knownblockversions;1";

/// FileData is an in memory representation of our integrity data
/// (see [KnownBlockVersions]).
/// It can be serialized to an actual file an deserialized from
/// an actual file.
#[derive(BinRead, BinWrite, Debug, PartialEq)]
pub struct FileData {
    #[binread(
        assert(header.iter().map(|c| c.get()).collect::<Vec<_>>() == FORMAT_VERSION_HEADER,
        "Wrong format version header: '{}'. Expected '{}'",
        format_potential_utf8(&header.iter().map(|c| c.get()).collect::<Vec<_>>()),
        format_potential_utf8(FORMAT_VERSION_HEADER)))]
    #[binread(parse_with = read_null_string)]
    #[binwrite(with(write_null_string))]
    header: Vec<NonZeroU8>,

    #[binread(parse_with = read_bool)]
    #[binwrite(with(write_bool))]
    pub integrity_violation_in_previous_run: bool,

    #[binread(parse_with = read_hashmap)]
    #[binwrite(with(write_hashmap))]
    pub known_block_versions: HashMap<(ClientId, BlockId), BlockVersion>,

    #[binread(parse_with = read_hashmap)]
    #[binwrite(with(write_hashmap))]
    pub last_update_client_id: HashMap<BlockId, ClientId>,
}

impl Default for FileData {
    fn default() -> FileData {
        let header = FORMAT_VERSION_HEADER
            .iter()
            .map(|&c| NonZeroU8::new(c).unwrap())
            .collect();
        FileData {
            header,
            integrity_violation_in_previous_run: false,
            known_block_versions: HashMap::new(),
            last_update_client_id: HashMap::new(),
        }
    }
}

fn format_potential_utf8(data: &[u8]) -> String {
    match std::str::from_utf8(data) {
        Ok(str) => String::from(str),
        Err(_) => format!("{{non-utf8 0x{}}}", hex::encode(data)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::binary::testutils::{binary, deserialize, test_serialize_deserialize};
    use common_macros::hash_map;

    #[test]
    fn given_wrong_header_utf8() {
        let error =
            deserialize::<FileData>(&binary(&[b"cryfs.integritydata.knownblockversions;20\0"]))
                .unwrap_err();
        if let binread::Error::AssertFail { pos, message } = error.downcast_ref().unwrap() {
            const EXPECTED_POS: u64 = 0;
            assert_eq!(
                0, EXPECTED_POS,
                "Expected to fail when reading the header at pos {} but failed at pos {}",
                EXPECTED_POS, pos
            );
            assert_eq!("Wrong format version header: 'cryfs.integritydata.knownblockversions;20'. Expected 'cryfs.integritydata.knownblockversions;1'", message);
        } else {
            panic!(
                "Expected to fail with AssertFail, but failed with {:?}",
                error
            );
        }
    }

    #[test]
    fn given_wrong_header_nonutf8() {
        let error = deserialize::<FileData>(&binary(&[b"cryfs\x80\0"])).unwrap_err();
        if let binread::Error::AssertFail { pos, message } = error.downcast_ref().unwrap() {
            const EXPECTED_POS: u64 = 0;
            assert_eq!(
                EXPECTED_POS, *pos,
                "Expected to fail when reading the header at pos {} but failed at pos {}",
                EXPECTED_POS, pos
            );
            assert_eq!("Wrong format version header: \'{non-utf8 0x637279667380}\'. Expected 'cryfs.integritydata.knownblockversions;1'", message);
        } else {
            panic!(
                "Expected to fail with AssertFail, but failed with {:?}",
                error
            );
        }
    }

    #[test]
    fn given_integrityviolationinpreviousrun_true() {
        test_serialize_deserialize(
            FileData {
                integrity_violation_in_previous_run: true,
                ..FileData::default()
            },
            &[&binary(&[
                FORMAT_VERSION_HEADER,
                b"\0",
                b"\x01",
                &0u64.to_le_bytes(),
                &0u64.to_le_bytes(),
            ])],
        );
    }

    #[test]
    fn given_integrityviolationinpreviousrun_false() {
        test_serialize_deserialize(
            FileData {
                integrity_violation_in_previous_run: false,
                ..FileData::default()
            },
            &[&binary(&[
                FORMAT_VERSION_HEADER,
                b"\0",
                b"\x00",
                &0u64.to_le_bytes(),
                &0u64.to_le_bytes(),
            ])],
        );
    }

    #[test]
    fn given_integrityviolationinpreviousrun_invalid() {
        let error = deserialize::<FileData>(&binary(&[
            FORMAT_VERSION_HEADER,
            b"\0",
            b"\x02",
            &0u64.to_le_bytes(),
            &0u64.to_le_bytes(),
        ]))
        .unwrap_err();

        if let binread::Error::AssertFail { pos, message } = error.downcast_ref().unwrap() {
            const EXPECTED_POS: u64 = 41;
            assert_eq!(EXPECTED_POS, *pos, "Expected to fail when reading integrity_violation_in_previous_run at pos {} but failed at pos {}", EXPECTED_POS, pos);
            assert_eq!(
                "Tried to read '2' as a boolean value. Must be 0 or 1.",
                message
            );
        } else {
            panic!(
                "Expected to fail with AssertFail, but failed with {:?}",
                error
            );
        }
    }

    #[test]
    fn given_knownblockversions_empty() {
        test_serialize_deserialize(
            FileData {
                known_block_versions: HashMap::new(),
                ..FileData::default()
            },
            &[&binary(&[
                FORMAT_VERSION_HEADER,
                b"\0",
                b"\x00",
                &0u64.to_le_bytes(),
                &0u64.to_le_bytes(),
            ])],
        );
    }

    #[test]
    fn given_knownblockversions_nonempty() {
        let first_entry = binary(&[
            &0x3ab74641u32.to_le_bytes(),
            &hex::decode("bd9cb3b508182dd71eda77c3ff99325c").unwrap(),
            &50u64.to_le_bytes(),
        ]);
        let second_entry = binary(&[
            &0x21233651u32.to_le_bytes(),
            &hex::decode("45fc5ad983c6c85a7a2859181d2199cb").unwrap(),
            &10_000_000u64.to_le_bytes(),
        ]);
        test_serialize_deserialize(
            FileData {
                known_block_versions: hash_map![
                    (ClientId{id:0x3ab74641}, BlockId::from_hex("bd9cb3b508182dd71eda77c3ff99325c").unwrap()) => BlockVersion{version:50},
                    (ClientId{id:0x21233651}, BlockId::from_hex("45fc5ad983c6c85a7a2859181d2199cb").unwrap()) => BlockVersion{version:10_000_000},
                ],
                ..FileData::default()
            },
            &[
                &binary(&[
                    FORMAT_VERSION_HEADER,
                    b"\0",
                    b"\x00",
                    &2u64.to_le_bytes(),
                    &first_entry,
                    &second_entry,
                    &0u64.to_le_bytes(),
                ]),
                &binary(&[
                    FORMAT_VERSION_HEADER,
                    b"\0",
                    b"\x00",
                    &2u64.to_le_bytes(),
                    &second_entry,
                    &first_entry,
                    &0u64.to_le_bytes(),
                ]),
            ],
        );
    }

    #[test]
    fn given_lastupdateclientid_empty() {
        test_serialize_deserialize(
            FileData {
                last_update_client_id: HashMap::new(),
                ..FileData::default()
            },
            &[&binary(&[
                FORMAT_VERSION_HEADER,
                b"\0",
                b"\x00",
                &0u64.to_le_bytes(),
                &0u64.to_le_bytes(),
            ])],
        );
    }

    #[test]
    fn given_lastupdateclientid_nonempty() {
        let first_entry = binary(&[
            &hex::decode("45fc5ad983c6c85a7a2859181d2199cb").unwrap(),
            &0x21233651u32.to_le_bytes(),
        ]);
        let second_entry = binary(&[
            &hex::decode("bd9cb3b508182dd71eda77c3ff99325c").unwrap(),
            &0x3ab74641u32.to_le_bytes(),
        ]);
        test_serialize_deserialize(
            FileData {
                last_update_client_id: hash_map![
                    BlockId::from_hex("bd9cb3b508182dd71eda77c3ff99325c").unwrap() => ClientId { id: 0x3ab74641 },
                    BlockId::from_hex("45fc5ad983c6c85a7a2859181d2199cb").unwrap() => ClientId { id: 0x21233651 },
                ],
                ..FileData::default()
            },
            &[
                &binary(&[
                    FORMAT_VERSION_HEADER,
                    b"\0",
                    b"\x00",
                    &0u64.to_le_bytes(),
                    &2u64.to_le_bytes(),
                    &first_entry,
                    &second_entry,
                ]),
                &binary(&[
                    FORMAT_VERSION_HEADER,
                    b"\0",
                    b"\x00",
                    &0u64.to_le_bytes(),
                    &2u64.to_le_bytes(),
                    &second_entry,
                    &first_entry,
                ]),
            ],
        );
    }

    #[test]
    fn given_knownblockversions_and_lastupdateclientid_nonempty() {
        test_serialize_deserialize(
            FileData {
                known_block_versions: hash_map![
                    (ClientId{id:0x3ab74641}, BlockId::from_hex("bd9cb3b508182dd71eda77c3ff99325c").unwrap()) => BlockVersion{version:50},
                ],
                last_update_client_id: hash_map![
                    BlockId::from_hex("bd9cb3b508182dd71eda77c3ff99325c").unwrap() => ClientId { id: 0x3ab74641 },
                ],
                ..FileData::default()
            },
            &[&binary(&[
                FORMAT_VERSION_HEADER,
                b"\0",
                b"\x00",
                // known_block_versions
                &1u64.to_le_bytes(),
                &0x3ab74641u32.to_le_bytes(),
                &hex::decode("bd9cb3b508182dd71eda77c3ff99325c").unwrap(),
                &50u64.to_le_bytes(),
                // last_update_client_id
                &1u64.to_le_bytes(),
                &hex::decode("bd9cb3b508182dd71eda77c3ff99325c").unwrap(),
                &0x3ab74641u32.to_le_bytes(),
            ])],
        );
    }

    #[test]
    fn given_toomuchdata() {
        let error = deserialize::<FileData>(&binary(&[
            FORMAT_VERSION_HEADER,
            b"\0",
            b"\x00",
            // known_block_versions
            &0u64.to_le_bytes(),
            &0u64.to_le_bytes(),
            b"some data left over",
        ]))
        .unwrap_err();
        assert_eq!(
            "After successfully reading, the stream still has 19 bytes left",
            error.to_string()
        );
    }
}
