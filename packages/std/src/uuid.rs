use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::str::FromStr;
use uuid as raw_uuid;

use crate::{from_json, to_json_vec};
use crate::{Api, Env, StdResult, Storage};

/// Uuid Provides a Uuid that can be used deterministically.
/// Use internally Uuidv5 and NAMESPACE_OID.
/// The name is combined with cahin id, contract address, block height, and increased sequential.
#[derive(
    Serialize, Deserialize, Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema,
)]
pub struct Uuid(#[schemars(with = "String")] raw_uuid::Uuid);
impl Uuid {
    pub fn as_slice(&self) -> &[u8] {
        &self.as_bytes()[0..16]
    }

    // Port the new_v5 implementation of uuid to use deps.api
    // https://github.com/uuid-rs/uuid/blob/2d6c147bdfca9612263dd7e82e26155f7ef8bf32/src/v5.rs#L33
    fn new_v5(api: &dyn Api, namespace: &Uuid, name: &[u8]) -> StdResult<Self> {
        let inputs = [namespace.as_bytes(), name];
        let buffer = api.sha1_calculate(&inputs)?;

        let mut bytes = raw_uuid::Bytes::default();
        bytes.copy_from_slice(&buffer[..16]);
        let mut builder = raw_uuid::Builder::from_bytes(bytes);
        builder
            .set_variant(raw_uuid::Variant::RFC4122)
            .set_version(raw_uuid::Version::Sha1);

        Ok(Uuid(builder.into_uuid()))
    }
}

const CONTRACT_UUID_SEQ_NUM_KEY: &[u8] = b"contract_uuid_seq_num";

pub fn new_uuid(env: &Env, storage: &mut dyn Storage, api: &dyn Api) -> StdResult<Uuid> {
    let raw_seq_num = storage.get(CONTRACT_UUID_SEQ_NUM_KEY);
    let seq_num: u16 = match raw_seq_num {
        Some(data) => from_json(data).unwrap(),
        None => 0,
    };
    let next_seq_num: u16 = seq_num.wrapping_add(1);

    let uuid_name = format!("{} {} {}", env.contract.address, env.block.height, seq_num);
    storage.set(
        CONTRACT_UUID_SEQ_NUM_KEY,
        &(to_json_vec(&next_seq_num).unwrap()),
    );

    Uuid::new_v5(
        api,
        &Uuid(raw_uuid::Uuid::NAMESPACE_OID),
        uuid_name.as_bytes(),
    )
}

impl Deref for Uuid {
    type Target = raw_uuid::Uuid;
    fn deref(&self) -> &raw_uuid::Uuid {
        &self.0
    }
}

impl FromStr for Uuid {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = raw_uuid::Uuid::parse_str(s);
        match parsed {
            Ok(data) => Ok(Uuid(data)),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::{mock_env, MockApi, MockStorage};
    use crate::{new_uuid, Uuid};
    use crate::{to_json_vec, Addr, Storage};
    use std::str::FromStr;
    use uuid as raw_uuid;

    const CONTRACT_UUID_SEQ_NUM_KEY: &[u8] = b"contract_uuid_seq_num";

    #[test]
    fn generate_uuid_v5() {
        let env = mock_env();
        let api = MockApi::default();
        let mut storage = MockStorage::new();

        let uuid = new_uuid(&env, &mut storage, &api).unwrap();
        let uuid2 = new_uuid(&env, &mut storage, &api).unwrap();

        assert_eq!(uuid.to_string(), "417d3461-5f6c-584b-8035-482a70997aee");
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
        assert_eq!(uuid.get_version(), Some(uuid::Version::Sha1));
        let parsed_uuid = Uuid::from_str("417d3461-5f6c-584b-8035-482a70997aee");
        assert_eq!(uuid, parsed_uuid.unwrap());

        assert_eq!(uuid2.to_string(), "61b5574c-d3e4-5d06-8a87-ab28a7353dfd");
        assert_ne!(uuid, uuid2);
    }

    #[test]
    fn same_output_as_raw_uuid() {
        let env = mock_env();
        let api = MockApi::default();
        let mut storage = MockStorage::new();
        let our_uuid = new_uuid(&env, &mut storage, &api).unwrap();

        let uuid_name = format!("{} {} {}", env.contract.address, env.block.height, 0);
        let raw = raw_uuid::Uuid::new_v5(&raw_uuid::Uuid::NAMESPACE_OID, uuid_name.as_bytes());

        assert_eq!(our_uuid.to_string(), raw.to_string());
    }

    #[test]
    fn work_fine_as_longest_name() {
        let mut env = mock_env();
        let api = MockApi::default();
        let mut storage = MockStorage::new();

        //enforce the max value
        env.contract.address = Addr::unchecked("link1qyqszqgpqyqszqgpqyqszqgpqyqszqgp8apuk5");
        env.block.height = u64::MAX;
        let stor: &mut dyn Storage = &mut storage;
        stor.set(
            CONTRACT_UUID_SEQ_NUM_KEY,
            &(to_json_vec(&u16::MAX).unwrap()),
        );

        let uuid = new_uuid(&env, &mut storage, &api);
        assert!(uuid.is_ok());
    }
}
