use uuid as raw_uuid;
use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::str::FromStr;

use crate::{Env, Storage, Api, StdResult};
use crate::{to_vec, from_slice};


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

const CONTRACT_UUID_SEQ_KEY: &[u8] = b"contract_uuid_seq";

pub fn new_uuid(env: &Env, storage: &mut dyn Storage, api: & dyn Api) -> StdResult<Uuid> {
    let raw_seq = storage.get(CONTRACT_UUID_SEQ_KEY);
    let seq: u64 = match raw_seq {
        Some(data) => {from_slice(&data).unwrap()},
        None => 0,
    };
    let next_seq: u64 = if seq < u64::MAX {
        seq +1
    } else {
        0
    };
    
    let uuid_name = format!("{} {} {} {}", env.block.chain_id, env.contract.address, env.block.height, seq);
    storage.set(CONTRACT_UUID_SEQ_KEY, &(to_vec(&next_seq).unwrap()));

    Uuid::new_v5(api, &Uuid(raw_uuid::Uuid::NAMESPACE_OID), uuid_name.as_bytes())
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
    use crate::testing::{mock_env, MockStorage, MockApi};
    use crate::{Uuid, new_uuid};
    use std::str::FromStr;

    #[test]
    fn generate_uuid_v5() {
        let env = mock_env();
        let api = MockApi::default();
        let mut storage = MockStorage::new();


        let uuid = new_uuid(&env, &mut storage, &api).unwrap();
        let uuid2 = new_uuid(&env, &mut storage, &api).unwrap();        
      
        assert_eq!(uuid.to_string(), "f448062e-7f17-5b6a-b683-1a6c01e0578f" );
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
        assert_eq!(uuid.get_version(), Some(uuid::Version::Sha1));
        let parsed_uuid = Uuid::from_str("f448062e-7f17-5b6a-b683-1a6c01e0578f");
        assert_eq!(uuid, parsed_uuid.unwrap());

        assert_eq!(uuid2.to_string(), "d45dfca2-dd58-543c-885c-8465cda0cff7" );
        assert_ne!(uuid, uuid2);
    }
}   