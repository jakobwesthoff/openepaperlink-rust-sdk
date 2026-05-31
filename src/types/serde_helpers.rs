// =========================================================
// Serde Helper Deserializers
// =========================================================

// The AP's JSON uses inconsistent representations across endpoints: capability
// flags are strings ("0"/"1") in get_ap_config but integers (0/1) in sysinfo,
// and apstate appears as both string "1" and integer 1 depending on the
// endpoint. These helpers handle the ambiguity.

use serde::{Deserialize, Deserializer, de};

/// Deserializes a string `"0"` or `"1"` into a `bool`.
///
/// Used for capability flags in the `get_ap_config` response, which are
/// always string-encoded despite being boolean values.
// Used by ApConfig (Phase 3) via #[serde(deserialize_with)]. Not referenced
// at the function level until then.
#[allow(dead_code)]
pub(crate) fn deserialize_string_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        other => Err(de::Error::custom(format!(
            "expected \"0\" or \"1\", got {other:?}"
        ))),
    }
}

/// Deserializes a value that may be either a string or an integer into a `u8`.
///
/// The AP sends `apstate` as a string (`"1"`) in the `get_ap_config` response
/// but as an integer (`1`) in the WebSocket `sys` heartbeat. This visitor
/// accepts either form.
pub(crate) fn deserialize_string_or_int_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrIntVisitor;

    impl<'de> de::Visitor<'de> for StringOrIntVisitor {
        type Value = u8;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a u8 integer or a string containing a u8 integer")
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<u8, E> {
            u8::try_from(v).map_err(|_| E::custom(format!("integer {v} out of u8 range")))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<u8, E> {
            v.parse()
                .map_err(|_| E::custom(format!("cannot parse {v:?} as u8")))
        }
    }

    deserializer.deserialize_any(StringOrIntVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_bool_zero_is_false() {
        let val: bool =
            deserialize_string_bool(serde_json::Value::String("0".into())).unwrap();
        assert!(!val);
    }

    #[test]
    fn string_bool_one_is_true() {
        let val: bool =
            deserialize_string_bool(serde_json::Value::String("1".into())).unwrap();
        assert!(val);
    }

    #[test]
    fn string_or_int_from_string() {
        let val: u8 =
            deserialize_string_or_int_u8(serde_json::Value::String("7".into())).unwrap();
        assert_eq!(val, 7);
    }

    #[test]
    fn string_or_int_from_int() {
        let val: u8 =
            deserialize_string_or_int_u8(serde_json::Value::Number(3.into())).unwrap();
        assert_eq!(val, 3);
    }
}

// serde_json::Value implements Deserializer, which makes it convenient for
// testing without constructing full JSON strings. The helpers above are
// designed to work with any Deserializer, not just serde_json::Value.
