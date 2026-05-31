use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::Error;

// =========================================================
// Mac Address Newtype
// =========================================================

// The AP encodes MAC addresses as uppercase hexadecimal strings with reversed
// byte order: the internal array stores bytes LSB-first (index 0 = least
// significant), while the wire format prints them MSB-first (index 7 first).
//
// 12-character inputs (6 bytes) are zero-padded at indices 6 and 7.
// The AP always emits 16-character strings.

/// An 8-byte MAC address as used by OpenEPaperLink tags.
///
/// Handles the reversed byte-order encoding that the AP uses on the wire.
/// Accepts both 12-character (6-byte) and 16-character (8-byte) hex strings
/// and always formats as a 16-character uppercase hex string.
///
/// # Examples
///
/// ```
/// use openepaperlink_sdk::Mac;
///
/// let mac: Mac = "00007E23907FB299".parse().unwrap();
/// assert_eq!(mac.to_string(), "00007E23907FB299");
///
/// // 12-character input is zero-padded at the high end
/// let short: Mac = "7E231842B297".parse().unwrap();
/// assert_eq!(short.to_string(), "00007E231842B297");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mac([u8; 8]);

impl Mac {
    /// Access the raw internal byte array.
    ///
    /// Index 0 is the least significant byte; index 7 is the most significant.
    /// This matches the AP's internal representation, not the wire hex string.
    pub fn bytes(&self) -> &[u8; 8] {
        &self.0
    }
}

impl FromStr for Mac {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(Error::InvalidMac {
                reason: "expected ASCII hex characters".to_string(),
            });
        }

        let hex_len = s.len();
        if hex_len != 12 && hex_len != 16 {
            return Err(Error::InvalidMac {
                reason: format!("expected 12 or 16 hex characters, got {hex_len}"),
            });
        }

        let byte_count = hex_len / 2;
        let mut parsed = [0u8; 8];

        for i in 0..byte_count {
            let hex_byte = &s[i * 2..i * 2 + 2];
            parsed[i] = u8::from_str_radix(hex_byte, 16).map_err(|_| Error::InvalidMac {
                reason: format!("invalid hex byte at position {}: {hex_byte:?}", i * 2),
            })?;
        }

        // The wire format prints bytes MSB-first: the first two hex chars are
        // the most significant byte. Internally we store LSB at index 0, so
        // we reverse the parsed bytes into the internal array.
        let mut mac = [0u8; 8];
        for i in 0..byte_count {
            mac[byte_count - 1 - i] = parsed[i];
        }
        // 12-char (6-byte) input: indices 6 and 7 stay zero (MSB padding).

        Ok(Mac(mac))
    }
}

impl fmt::Display for Mac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print bytes in reverse order (index 7 first) to produce the wire
        // format: MSB-first uppercase hex.
        for i in (0..8).rev() {
            write!(f, "{:02X}", self.0[i])?;
        }
        Ok(())
    }
}

impl fmt::Debug for Mac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mac({self})")
    }
}

impl Serialize for Mac {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Mac {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_16_char_mac() {
        let mac: Mac = "6F5E4D3C2B1A0000".parse().unwrap();
        assert_eq!(
            *mac.bytes(),
            [0x00, 0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F]
        );
    }

    #[test]
    fn parse_12_char_mac() {
        let mac: Mac = "7E231842B297".parse().unwrap();
        // 6 parsed bytes reversed into indices 0..5, indices 6,7 = 0
        assert_eq!(
            *mac.bytes(),
            [0x97, 0xB2, 0x42, 0x18, 0x23, 0x7E, 0x00, 0x00]
        );
        assert_eq!(mac.to_string(), "00007E231842B297");
    }

    #[test]
    fn display_matches_protocol_doc_example() {
        // Protocol doc: internal [0x00,0x00,0x1A,0x2B,0x3C,0x4D,0x5E,0x6F]
        // → "6F5E4D3C2B1A0000"
        let mac = Mac([0x00, 0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F]);
        assert_eq!(mac.to_string(), "6F5E4D3C2B1A0000");
    }

    #[test]
    fn round_trip_16_char() {
        let input = "00007E23907FB299";
        let mac: Mac = input.parse().unwrap();
        assert_eq!(mac.to_string(), input);
    }

    #[test]
    fn round_trip_serde() {
        let mac: Mac = "00007E23907FB299".parse().unwrap();
        let json = serde_json::to_string(&mac).unwrap();
        assert_eq!(json, "\"00007E23907FB299\"");

        let deserialized: Mac = serde_json::from_str(&json).unwrap();
        assert_eq!(mac, deserialized);
    }

    #[test]
    fn reject_invalid_length() {
        assert!("AABBCC".parse::<Mac>().is_err());
        assert!("AABBCCDDEEFF00112233".parse::<Mac>().is_err());
    }

    #[test]
    fn reject_invalid_hex() {
        assert!("GGHHIIJJKKLL".parse::<Mac>().is_err());
        assert!("00007E23907FBX99".parse::<Mac>().is_err());
    }

    #[test]
    fn reject_multibyte_utf8_without_panic() {
        // "€€€€" is 12 bytes but not ASCII — must not panic on slice
        assert!("€€€€".parse::<Mac>().is_err());
    }

    #[test]
    fn case_insensitive_parse() {
        let upper: Mac = "00007E23907FB299".parse().unwrap();
        let lower: Mac = "00007e23907fb299".parse().unwrap();
        assert_eq!(upper, lower);
    }
}
