# Unify capability flag types across ApConfig and SystemInfo

`ApConfig` deserializes capability flags (`has_c6`, `has_h2`, etc.) as `bool`
via `deserialize_string_bool`, while `SystemInfo` uses `u8` for the same
fields. Both represent boolean values on the wire (just encoded differently:
`"1"` vs `1`). `SystemInfo` should also use `bool` for consistency.

Similarly, `SaveTagConfig.invert` is `Option<u8>` while
`UploadImageOptions.invert` is `Option<bool>` — both represent the same
boolean concept.
