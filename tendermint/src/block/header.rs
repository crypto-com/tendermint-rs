//! Block headers
use amino_types::{message::AminoMessage, BlockId, ConsensusVersion, TimeMsg};
use serde::{Deserialize, Serialize};

use crate::merkle::simple_hash_from_byte_slices;
use crate::{account, amino_types, block, chain, lite, serializers, Hash, Time};

/// Block `Header` values contain metadata about the block and about the
/// consensus, as well as commitments to the data in the current block, the
/// previous block, and the results returned by the application.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#header>
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Header {
    /// Header version
    pub version: Version,

    /// Chain ID
    pub chain_id: chain::Id,

    /// Current block height
    pub height: block::Height,

    /// Current timestamp
    pub time: Time,

    /// Number of transactions in block
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub num_txs: u64,

    /// Total number of transactions
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub total_txs: u64,

    /// Previous block info
    #[serde(deserialize_with = "serializers::parse_non_empty_block_id")]
    pub last_block_id: Option<block::Id>,

    /// Commit from validators from the last block
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub last_commit_hash: Option<Hash>,

    /// Merkle root of transaction hashes
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub data_hash: Option<Hash>,

    /// Validators for the current block
    pub validators_hash: Hash,

    /// Validators for the next block
    pub next_validators_hash: Hash,

    /// Consensus params for the current block
    pub consensus_hash: Hash,

    /// State after txs from the previous block
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub app_hash: Option<Hash>,

    /// Root hash of all results from the txs from the previous block
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub last_results_hash: Option<Hash>,

    /// Hash of evidence included in the block
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub evidence_hash: Option<Hash>,

    /// Original proposer of the block
    pub proposer_address: account::Id,
}

impl lite::Header for Header {
    fn height(&self) -> block::Height {
        self.height
    }

    fn bft_time(&self) -> Time {
        self.time
    }

    fn validators_hash(&self) -> Hash {
        self.validators_hash
    }

    fn next_validators_hash(&self) -> Hash {
        self.next_validators_hash
    }

    fn hash(&self) -> Hash {
        // Note that if there is an encoding problem this will
        // panic (as the golang code would):
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/block.go#L393
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/encoding_helper.go#L9:6

        let mut byteslices: Vec<Vec<u8>> = Vec::with_capacity(16);
        byteslices.push(AminoMessage::bytes_vec(&ConsensusVersion::from(
            &self.version,
        )));
        byteslices.push(bytes_enc(self.chain_id.as_bytes()));
        byteslices.push(encode_varint(self.height.value()));
        byteslices.push(AminoMessage::bytes_vec(&TimeMsg::from(self.time)));
        byteslices.push(encode_varint(self.num_txs));
        byteslices.push(encode_varint(self.total_txs));
        byteslices.push(
            self.last_block_id
                .as_ref()
                .map_or(vec![], |id| AminoMessage::bytes_vec(&BlockId::from(id))),
        );
        byteslices.push(self.last_commit_hash.as_ref().map_or(vec![], encode_hash));
        byteslices.push(self.data_hash.as_ref().map_or(vec![], encode_hash));
        byteslices.push(encode_hash(&self.validators_hash));
        byteslices.push(encode_hash(&self.next_validators_hash));
        byteslices.push(encode_hash(&self.consensus_hash));
        byteslices.push(self.app_hash.as_ref().map_or(vec![], encode_hash));
        byteslices.push(self.last_results_hash.as_ref().map_or(vec![], encode_hash));
        byteslices.push(self.evidence_hash.as_ref().map_or(vec![], encode_hash));
        byteslices.push(bytes_enc(self.proposer_address.as_bytes()));

        Hash::Sha256(simple_hash_from_byte_slices(byteslices))
    }
}

/// `Version` contains the protocol version for the blockchain and the
/// application.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#version>
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Version {
    /// Block version
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub block: u64,

    /// App version
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub app: u64,
}

fn bytes_enc(bytes: &[u8]) -> Vec<u8> {
    let mut chain_id_enc = vec![];
    prost_amino::encode_length_delimiter(bytes.len(), &mut chain_id_enc).unwrap();
    chain_id_enc.append(&mut bytes.to_vec());
    chain_id_enc
}

fn encode_hash(hash: &Hash) -> Vec<u8> {
    bytes_enc(hash.as_bytes())
}

fn encode_varint(val: u64) -> Vec<u8> {
    let mut val_enc = vec![];
    prost_amino::encoding::encode_varint(val, &mut val_enc);
    val_enc
}
