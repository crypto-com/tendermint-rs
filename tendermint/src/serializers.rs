//! Serde serializers

use crate::{block, Hash};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use std::time::Duration;

/// Parse `i64` from a JSON string
pub(crate) fn parse_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<i64>()
        .map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize `i64` as a JSON string
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_i64<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", value).serialize(serializer)
}

/// Parse `u64` from a JSON string
pub(crate) fn parse_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize `u64` as a JSON string
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_u64<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", value).serialize(serializer)
}

/// Parse `Duration` from a JSON string containing a nanosecond count
pub(crate) fn parse_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    // TODO(tarcieri): handle 64-bit overflow?
    let nanos = String::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(|e| D::Error::custom(format!("{}", e)))?;

    Ok(Duration::from_nanos(nanos))
}

/// Serialize `Duration` as a JSON string containing a nanosecond count
pub(crate) fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", duration.as_nanos()).serialize(serializer)
}

/// Parse `Height` from json, `None` if value is invalid.
pub(crate) fn parse_height_option<'de, D>(
    deserializer: D,
) -> Result<Option<block::Height>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?
        .parse::<block::Height>()
        .ok())
}

///
pub(crate) fn parse_non_empty_hash<'de, D>(deserializer: D) -> Result<Option<Hash>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Option::deserialize(deserializer)?;
    match o.filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) => Ok(Some(
            Hash::from_str(&s).map_err(|err| D::Error::custom(format!("{}", err)))?,
        )),
    }
}

/// Parse empty block id as None.
pub(crate) fn parse_non_empty_block_id<'de, D>(
    deserializer: D,
) -> Result<Option<block::Id>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Parts {
        #[serde(deserialize_with = "parse_u64")]
        total: u64,
        hash: String,
    }
    #[derive(Deserialize)]
    struct BlockId {
        hash: String,
        parts: Parts,
    }
    if let Some(tmp_id) = <Option<BlockId>>::deserialize(deserializer)? {
        if tmp_id.hash.is_empty() {
            Ok(None)
        } else {
            Ok(Some(block::Id {
                hash: Hash::from_str(&tmp_id.hash)
                    .map_err(|err| D::Error::custom(format!("{}", err)))?,
                parts: if tmp_id.parts.hash.is_empty() {
                    None
                } else {
                    Some(block::parts::Header {
                        total: tmp_id.parts.total,
                        hash: Hash::from_str(&tmp_id.parts.hash)
                            .map_err(|err| D::Error::custom(format!("{}", err)))?,
                    })
                },
            }))
        }
    } else {
        Ok(None)
    }
}
