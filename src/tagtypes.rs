//! Tag type descriptors: fetching the display hardware definition for a tag's
//! `hwType`.

use crate::client::Client;
use crate::{Error, TagType};

impl Client {
    /// Fetch the [`TagType`] descriptor for a hardware type ID.
    ///
    /// The `hw_type` is the [`hw_type`](crate::TagRecord::hw_type) byte of a
    /// tag record. The descriptor defines the display's dimensions and color
    /// palette. An unknown hardware type yields an HTTP 404, surfaced as
    /// [`Error::Http`].
    pub async fn get_tag_type(&self, hw_type: u8) -> Result<TagType, Error> {
        let url = self.url(&tagtype_path(hw_type));
        let descriptor: TagType = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(descriptor)
    }
}

/// Build the descriptor path for a hardware type: `/tagtypes/<HH>.json`, where
/// `<HH>` is the uppercase, zero-padded two-digit hex of the type ID.
fn tagtype_path(hw_type: u8) -> String {
    format!("/tagtypes/{hw_type:02X}.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_path_with_uppercase_padded_hex() {
        assert_eq!(tagtype_path(0x33), "/tagtypes/33.json");
        assert_eq!(tagtype_path(0x00), "/tagtypes/00.json");
        assert_eq!(tagtype_path(0xFF), "/tagtypes/FF.json");
        // Single hex digit must be zero-padded and uppercased.
        assert_eq!(tagtype_path(0x0A), "/tagtypes/0A.json");
    }
}
