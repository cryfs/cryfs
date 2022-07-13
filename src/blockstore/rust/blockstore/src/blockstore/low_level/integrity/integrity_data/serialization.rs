use binread::BinRead;
use binwrite::BinWrite;
use core::num::NonZeroU8;
use lockable::LockableHashMap;
use std::collections::hash_map::HashMap;
use std::sync::Arc;

use crate::blockstore::BlockId;
use crate::utils::binary::{
    read_bool, read_hashmap, read_null_string, write_bool, write_hashmap, write_null_string,
};
use crate::utils::containers::HashMapExt;

use super::known_block_versions::{
    BlockInfo, BlockVersion, ClientId, KnownBlockVersions, MaybeClientId,
};

const FORMAT_VERSION_HEADER: &[u8] = b"cryfs.integritydata.knownblockversions;1";

// TODO Is KnownBlockVersionsSerialized compatible with the C++ version?

/// KnownBlockVersionsSerialized is an in memory representation of our integrity data
/// (see [KnownBlockVersions]).
/// It can be serialized to an actual file and deserialized from
/// an actual file.
#[derive(BinRead, BinWrite, Debug, PartialEq)]
pub struct KnownBlockVersionsSerialized {
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
    pub last_update_client_id: HashMap<BlockId, MaybeClientId>,
}

// TODO Test
impl From<KnownBlockVersionsSerialized> for KnownBlockVersions {
    fn from(data: KnownBlockVersionsSerialized) -> KnownBlockVersions {
        // TODO A better algorithm may be to create a HashMap first and then use .into_iter().collect() to get it into a LockableHashMap
        let block_infos = LockableHashMap::new();
        // TODO block_infos.reserve(data.last_update_client_id.len());
        for (block_id, client_id) in data.last_update_client_id {
            let mut block_info = block_infos.try_lock(block_id).expect(
                "We're just creating this object, nobody else has access. Locking can't fail",
            );
            block_info
                .try_insert(BlockInfo::new_unknown(client_id))
                .expect("Input hashmap last_update_client_id had duplicate keys");
        }
        for ((client_id, block_id), block_version) in data.known_block_versions {
            let mut block_info = block_infos.try_lock(block_id).expect(
                "We're just creating this object, nobody else has access. Locking can't fail",
            );
            let block_info = block_info.value_or_insert_with(|| {
                BlockInfo::new_unknown(MaybeClientId::ClientId(client_id))
            });
            HashMapExt::try_insert(
                &mut block_info.known_block_versions,
                client_id,
                block_version,
            )
            .expect("Input hashmap had duplicate keys");
        }
        let result = KnownBlockVersions {
            integrity_violation_in_previous_run: data.integrity_violation_in_previous_run.into(),
            block_infos: Arc::new(block_infos),
        };
        result
    }
}

impl KnownBlockVersionsSerialized {
    pub async fn async_from(data: KnownBlockVersions) -> Self {
        let mut known_block_versions = HashMap::new();
        let mut last_update_client_id = HashMap::new();
        let num_blocks = data.block_infos.num_entries_or_locked();
        known_block_versions.reserve(num_blocks);
        last_update_client_id.reserve(num_blocks);

        let integrity_violation_in_previous_run = data.integrity_violation_in_previous_run();

        // At this point, we have exclusive access to KnownBlockVersions, but there may still be other threads/tasks having access to
        // a clone of the Arc containing KnownBlockVersions.block_infos. The only way for them to hold such a copy is through a
        // HashMapOwnedGuard that locks one of the block ids. HashMapOwnedGuard cannot be cloned. It is reasonable to assume that
        // these threads/tasks will at some point release their lock. Since we have exclusive access to the KnownBlockVersions object,
        // we know that no new threads/tasks can acquire such a lock. We just have to wait until the last thread/task releases their lock.
        // However, note that there is still a chance of a deadlock here. If one of those threads is the current thread, or if one of
        // those threads waits for the current thread on something, then we have a deadlock.
        // TODO Is there a better way to handle this?
        while Arc::strong_count(&data.block_infos) > 1 {
            // TODO Is there a better alternative that doesn't involve busy waiting?
            tokio::task::yield_now().await;
        }
        let block_infos = Arc::try_unwrap(data.block_infos).expect("We just waited until we're the only one with a clone of the Arc. It can't have gone back up.");

        for (block_id, block_info) in block_infos.into_entries_unordered() {
            for (client_id, block_version) in block_info.known_block_versions {
                HashMapExt::try_insert(
                    &mut known_block_versions,
                    (client_id, block_id),
                    block_version,
                )
                .expect("Input hashmap had duplicate keys");
            }
            HashMapExt::try_insert(
                &mut last_update_client_id,
                block_id,
                block_info.last_update_client_id,
            )
            .expect("Input hashmap had duplicate keys");
        }
        Self {
            header: FORMAT_VERSION_HEADER
                .iter()
                .map(|&c| NonZeroU8::new(c).unwrap())
                .collect(),
            integrity_violation_in_previous_run,
            known_block_versions,
            last_update_client_id,
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
    use super::super::known_block_versions::KnownBlockVersions;
    use super::*;
    use crate::utils::binary::testutils::{binary, deserialize, test_serialize_deserialize};
    use common_macros::hash_map;
    use std::num::NonZeroU32;

    async fn known_block_versions_serialized_default() -> KnownBlockVersionsSerialized {
        KnownBlockVersionsSerialized::async_from(KnownBlockVersions::default()).await
    }

    #[test]
    fn given_wrong_header_utf8() {
        let error = deserialize::<KnownBlockVersionsSerialized>(&binary(&[
            b"cryfs.integritydata.knownblockversions;20\0",
        ]))
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
        let error =
            deserialize::<KnownBlockVersionsSerialized>(&binary(&[b"cryfs\x80\0"])).unwrap_err();
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

    #[tokio::test]
    async fn given_integrityviolationinpreviousrun_true() {
        test_serialize_deserialize(
            KnownBlockVersionsSerialized {
                integrity_violation_in_previous_run: true,
                ..known_block_versions_serialized_default().await
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

    #[tokio::test]
    async fn given_integrityviolationinpreviousrun_false() {
        test_serialize_deserialize(
            KnownBlockVersionsSerialized {
                integrity_violation_in_previous_run: false,
                ..known_block_versions_serialized_default().await
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
        let error = deserialize::<KnownBlockVersionsSerialized>(&binary(&[
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

    #[tokio::test]
    async fn given_knownblockversions_empty() {
        test_serialize_deserialize(
            KnownBlockVersionsSerialized {
                known_block_versions: HashMap::new(),
                ..known_block_versions_serialized_default().await
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

    #[tokio::test]
    async fn given_knownblockversions_nonempty() {
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
            KnownBlockVersionsSerialized {
                known_block_versions: hash_map![
                    (ClientId{id:NonZeroU32::new(0x3ab74641).unwrap()}, BlockId::from_hex("bd9cb3b508182dd71eda77c3ff99325c").unwrap()) => BlockVersion{version:50},
                    (ClientId{id:NonZeroU32::new(0x21233651).unwrap()}, BlockId::from_hex("45fc5ad983c6c85a7a2859181d2199cb").unwrap()) => BlockVersion{version:10_000_000},
                ],
                ..known_block_versions_serialized_default().await
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

    #[tokio::test]
    async fn given_lastupdateclientid_empty() {
        test_serialize_deserialize(
            KnownBlockVersionsSerialized {
                last_update_client_id: HashMap::new(),
                ..known_block_versions_serialized_default().await
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

    #[tokio::test]
    async fn given_lastupdateclientid_nonempty() {
        let first_entry = binary(&[
            &hex::decode("45fc5ad983c6c85a7a2859181d2199cb").unwrap(),
            &0x21233651u32.to_le_bytes(),
        ]);
        let second_entry = binary(&[
            &hex::decode("bd9cb3b508182dd71eda77c3ff99325c").unwrap(),
            &0x3ab74641u32.to_le_bytes(),
        ]);
        test_serialize_deserialize(
            KnownBlockVersionsSerialized {
                last_update_client_id: hash_map![
                    BlockId::from_hex("bd9cb3b508182dd71eda77c3ff99325c").unwrap() => MaybeClientId::ClientId(ClientId { id: NonZeroU32::new(0x3ab74641).unwrap() }),
                    BlockId::from_hex("45fc5ad983c6c85a7a2859181d2199cb").unwrap() => MaybeClientId::ClientId(ClientId { id: NonZeroU32::new(0x21233651).unwrap() }),
                ],
                ..known_block_versions_serialized_default().await
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

    #[tokio::test]
    async fn given_knownblockversions_and_lastupdateclientid_nonempty() {
        test_serialize_deserialize(
            KnownBlockVersionsSerialized {
                known_block_versions: hash_map![
                    (ClientId{id:NonZeroU32::new(0x3ab74641).unwrap()}, BlockId::from_hex("bd9cb3b508182dd71eda77c3ff99325c").unwrap()) => BlockVersion{version:50},
                ],
                last_update_client_id: hash_map![
                    BlockId::from_hex("bd9cb3b508182dd71eda77c3ff99325c").unwrap() => MaybeClientId::ClientId(ClientId { id: NonZeroU32::new(0x3ab74641).unwrap() }),
                ],
                ..known_block_versions_serialized_default().await
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
        let error = deserialize::<KnownBlockVersionsSerialized>(&binary(&[
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
