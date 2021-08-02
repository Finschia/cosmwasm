/// Ostracon protobuf version

/// Ostracon repository URL.
pub const OSTRACON_REPO: &str = "https://github.com/line/ostracon";
// Commitish formats:
// Tag: v0.34.0-rc4
// Branch: master
// Commit ID (full length): d7d0ffea13c60c98b812d243ba5a2c375f341c15
pub const OSTRACON_COMMITISH: &str = "a727fe1db4e7e7f7037e15b61b76bab2d01de829";

/// Predefined custom attributes for message annotations
const PRIMITIVE_ENUM: &str = r#"#[derive(::num_derive::FromPrimitive, ::num_derive::ToPrimitive)]"#;
const SERIALIZED: &str = r#"#[derive(::serde::Deserialize, ::serde::Serialize)]"#;
const TYPE_TAG: &str = r#"#[serde(tag = "type", content = "value")]"#;

/// Predefined custom attributes for field annotations
const QUOTED: &str = r#"#[serde(with = "crate::serializers::from_str")]"#;
const QUOTED_WITH_DEFAULT: &str = r#"#[serde(with = "crate::serializers::from_str", default)]"#;
const HEXSTRING: &str = r#"#[serde(with = "crate::serializers::bytes::hexstring")]"#;
const BASE64STRING: &str = r#"#[serde(with = "crate::serializers::bytes::base64string")]"#;
const VEC_BASE64STRING: &str = r#"#[serde(with = "crate::serializers::bytes::vec_base64string")]"#;
const OPTIONAL: &str = r#"#[serde(with = "crate::serializers::optional")]"#;
const VEC_SKIP_IF_EMPTY: &str =
    r#"#[serde(skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]"#;
const NULLABLEVECARRAY: &str = r#"#[serde(with = "crate::serializers::txs")]"#;
const NULLABLE: &str = r#"#[serde(with = "crate::serializers::nullable")]"#;
const ALIAS_POWER_QUOTED: &str =
    r#"#[serde(alias = "power", with = "crate::serializers::from_str")]"#;
const PART_SET_HEADER_TOTAL: &str =
    r#"#[serde(with = "crate::serializers::part_set_header_total")]"#;
const RENAME_EDPUBKEY: &str = r#"#[serde(rename = "ostracon/PubKeyEd25519", with = "crate::serializers::bytes::base64string")]"#;
const RENAME_SECPPUBKEY: &str = r#"#[serde(rename = "ostracon/PubKeySecp256k1", with = "crate::serializers::bytes::base64string")]"#;
const RENAME_DUPLICATEVOTE: &str = r#"#[serde(rename = "ostracon/DuplicateVoteEvidence")]"#;
const RENAME_LIGHTCLIENTATTACK: &str =
    r#"#[serde(rename = "ostracon/LightClientAttackEvidence")]"#;
const EVIDENCE_VARIANT: &str = r#"#[serde(from = "crate::serializers::evidence::EvidenceVariant", into = "crate::serializers::evidence::EvidenceVariant")]"#;
const ALIAS_PARTS: &str = r#"#[serde(alias = "parts")]"#;

/// Custom type attributes applied on top of protobuf structs
/// The first item in the tuple defines the message where the annotation should apply and
/// the second item is the string that should be added as annotation.
/// The first item is a path as defined in the prost_build::Config::btree_map here:
/// https://docs.rs/prost-build/0.6.1/prost_build/struct.Config.html#method.btree_map
pub static CUSTOM_TYPE_ATTRIBUTES: &[(&str, &str)] = &[
    (".ostracon.libs.bits.BitArray", SERIALIZED),
    (".ostracon.types.EvidenceParams", SERIALIZED),
    (".ostracon.types.BlockIDFlag", PRIMITIVE_ENUM),
    (".ostracon.types.Block", SERIALIZED),
    (".ostracon.types.Data", SERIALIZED),
    (".ostracon.types.EvidenceList", SERIALIZED),
    (".ostracon.types.Evidence", SERIALIZED),
    (".ostracon.types.DuplicateVoteEvidence", SERIALIZED),
    (".ostracon.types.Vote", SERIALIZED),
    (".ostracon.types.BlockID", SERIALIZED),
    (".ostracon.types.PartSetHeader", SERIALIZED),
    (".ostracon.types.LightClientAttackEvidence", SERIALIZED),
    (".ostracon.types.LightBlock", SERIALIZED),
    (".ostracon.types.SignedHeader", SERIALIZED),
    (".ostracon.types.Header", SERIALIZED),
    (".ostracon.version.Consensus", SERIALIZED),
    (".ostracon.types.Commit", SERIALIZED),
    (".ostracon.types.CommitSig", SERIALIZED),
    (".ostracon.types.VoterSet", SERIALIZED),
    (".ostracon.types.ValidatorSet", SERIALIZED),
    (".ostracon.crypto.PublicKey", SERIALIZED),
    (".ostracon.crypto.PublicKey.sum", TYPE_TAG),
    (".ostracon.types.Evidence.sum", TYPE_TAG),
    (".ostracon.abci.ResponseInfo", SERIALIZED),
    (".ostracon.types.CanonicalBlockID", SERIALIZED),
    (".ostracon.types.CanonicalPartSetHeader", SERIALIZED),
    (".ostracon.types.Validator", SERIALIZED),
    (".ostracon.types.CanonicalVote", SERIALIZED),
    (".ostracon.types.BlockMeta", SERIALIZED),
    (".ostracon.types.Evidence", EVIDENCE_VARIANT),
    (".ostracon.types.TxProof", SERIALIZED),
    (".ostracon.crypto.CompositePublicKey", SERIALIZED),
    (".ostracon.crypto.Proof", SERIALIZED),
];

/// Custom field attributes applied on top of protobuf fields in (a) struct(s)
/// The first item in the tuple defines the field where the annotation should apply and
/// the second item is the string that should be added as annotation.
/// The first item is a path as defined in the prost_build::Config::btree_map here:
/// https://docs.rs/prost-build/0.6.1/prost_build/struct.Config.html#method.btree_map
pub static CUSTOM_FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    (
        ".ostracon.types.EvidenceParams.max_bytes",
        QUOTED_WITH_DEFAULT,
    ),
    (".ostracon.abci.ResponseInfo.last_block_height", QUOTED),
    (".ostracon.version.Consensus.block", QUOTED),
    (".ostracon.version.Consensus.app", QUOTED_WITH_DEFAULT),
    (
        ".ostracon.abci.ResponseInfo.last_block_app_hash",
        VEC_SKIP_IF_EMPTY,
    ),
    (".ostracon.abci.ResponseInfo.app_version", QUOTED),
    (".ostracon.types.BlockID.hash", HEXSTRING),
    (".ostracon.types.BlockID.part_set_header", ALIAS_PARTS),
    (
        ".ostracon.types.CanonicalBlockID.part_set_header",
        ALIAS_PARTS,
    ),
    (
        ".ostracon.types.PartSetHeader.total",
        PART_SET_HEADER_TOTAL,
    ),
    (".ostracon.types.PartSetHeader.hash", HEXSTRING),
    (".ostracon.types.Header.height", QUOTED),
    (".ostracon.types.Header.time", OPTIONAL),
    (".ostracon.types.Header.last_commit_hash", HEXSTRING),
    (".ostracon.types.Header.data_hash", HEXSTRING),
    (".ostracon.types.Header.validators_hash", HEXSTRING),
    (".ostracon.types.Header.next_validators_hash", HEXSTRING),
    (".ostracon.types.Header.consensus_hash", HEXSTRING),
    (".ostracon.types.Header.app_hash", HEXSTRING),
    (".ostracon.types.Header.last_results_hash", HEXSTRING),
    (".ostracon.types.Header.evidence_hash", HEXSTRING),
    (".ostracon.types.Header.proposer_address", HEXSTRING),
    (".ostracon.types.Data.txs", NULLABLEVECARRAY),
    (".ostracon.types.EvidenceList.evidence", NULLABLE),
    (".ostracon.types.Commit.height", QUOTED),
    (".ostracon.types.Commit.signatures", NULLABLE),
    (".ostracon.types.CommitSig.validator_address", HEXSTRING),
    (".ostracon.types.CommitSig.timestamp", OPTIONAL),
    (".ostracon.types.CommitSig.signature", BASE64STRING),
    (".ostracon.types.Vote.round", QUOTED),
    (".ostracon.types.Vote.height", QUOTED),
    (".ostracon.types.Vote.validator_index", QUOTED),
    (".ostracon.types.Vote.validator_address", HEXSTRING),
    (".ostracon.types.Vote.signature", BASE64STRING),
    (".ostracon.types.Vote.timestamp", OPTIONAL),
    (".ostracon.types.Validator.address", HEXSTRING),
    (
        ".ostracon.types.Validator.voting_power",
        ALIAS_POWER_QUOTED,
    ), // https://github.com/tendermint/tendermint/issues/5549
    (
        ".ostracon.types.Validator.proposer_priority",
        QUOTED_WITH_DEFAULT,
    ), // Default is for /genesis deserialization
    (".ostracon.types.BlockMeta.block_size", QUOTED),
    (".ostracon.types.BlockMeta.num_txs", QUOTED),
    (".ostracon.crypto.PublicKey.sum.ed25519", RENAME_EDPUBKEY),
    (".ostracon.crypto.PublicKey.sum.secp256k1", RENAME_SECPPUBKEY),
    (
        ".ostracon.types.Evidence.sum.duplicate_vote_evidence",
        RENAME_DUPLICATEVOTE,
    ),
    (
        ".ostracon.types.Evidence.sum.light_client_attack_evidence",
        RENAME_LIGHTCLIENTATTACK,
    ),
    (".ostracon.types.TxProof.data", BASE64STRING),
    (".ostracon.types.TxProof.root_hash", HEXSTRING),
    (".ostracon.crypto.Proof.index", QUOTED),
    (".ostracon.crypto.Proof.total", QUOTED),
    (".ostracon.crypto.Proof.aunts", VEC_BASE64STRING),
    (".ostracon.crypto.Proof.leaf_hash", BASE64STRING),
];
