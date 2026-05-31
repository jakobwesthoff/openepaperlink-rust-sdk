# ApConfig serialize does not round-trip string booleans

`ApConfig` deserializes capability flags from `"0"`/`"1"` strings via
`deserialize_string_bool`, but the derived `Serialize` emits `true`/`false`.
If `ApConfig` were ever serialized and sent back to the AP, the AP would
reject the boolean format. Currently not a problem since `SaveApConfig` is
the write type, but the asymmetry is worth noting for future API consumers.
