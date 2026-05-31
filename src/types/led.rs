use crate::Error;

/// A 12-byte LED flash pattern for tag LED control.
///
/// The pattern is sent as a 24-character hex string to `GET /led_flash`.
/// See the protocol documentation for the byte layout (mode, colors,
/// flash counts, delays, repeats).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedFlashPattern([u8; 12]);

impl LedFlashPattern {
    /// Create a pattern from raw bytes.
    pub fn from_bytes(bytes: [u8; 12]) -> Self {
        Self(bytes)
    }

    /// Parse a 24-character hex string into a pattern.
    pub fn from_hex(hex: &str) -> Result<Self, Error> {
        if !hex.is_ascii() {
            return Err(Error::InvalidLedPattern {
                reason: "expected ASCII hex characters".to_string(),
            });
        }

        if hex.len() != 24 {
            return Err(Error::InvalidLedPattern {
                reason: format!("expected 24 hex characters, got {}", hex.len()),
            });
        }

        let mut bytes = [0u8; 12];
        for i in 0..12 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).map_err(|_| {
                Error::InvalidLedPattern {
                    reason: format!("invalid hex at position {}", i * 2),
                }
            })?;
        }
        Ok(Self(bytes))
    }

    /// Format the pattern as a 24-character uppercase hex string.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02X}")).collect()
    }

    /// A pattern that stops any active LED flashing (all zeros).
    pub fn stop() -> Self {
        Self([0u8; 12])
    }

    /// Access the raw bytes.
    pub fn bytes(&self) -> &[u8; 12] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hex_valid() {
        let pattern = LedFlashPattern::from_hex("2120530A20530A20530A0A00").unwrap();
        assert_eq!(pattern.bytes()[0], 0x21);
        assert_eq!(pattern.bytes()[11], 0x00);
    }

    #[test]
    fn to_hex_round_trip() {
        let input = "2120530A20530A20530A0A00";
        let pattern = LedFlashPattern::from_hex(input).unwrap();
        assert_eq!(pattern.to_hex(), input);
    }

    #[test]
    fn reject_wrong_length() {
        assert!(LedFlashPattern::from_hex("AABB").is_err());
        assert!(LedFlashPattern::from_hex("2120530A20530A20530A0A0000").is_err());
    }

    #[test]
    fn reject_invalid_hex() {
        assert!(LedFlashPattern::from_hex("GGHHIIJJKKLLMMNNOOPP0000").is_err());
    }

    #[test]
    fn reject_multibyte_utf8_without_panic() {
        // "€€€€€€€€" is 24 bytes but not ASCII — must not panic on slice
        assert!(LedFlashPattern::from_hex("€€€€€€€€").is_err());
    }

    #[test]
    fn stop_pattern_is_all_zeros() {
        let stop = LedFlashPattern::stop();
        assert_eq!(stop.to_hex(), "000000000000000000000000");
    }
}
