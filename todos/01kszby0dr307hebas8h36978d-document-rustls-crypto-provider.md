# Document rustls crypto provider requirement

The `rustls` feature flag activates `reqwest/rustls` which may require
a crypto provider (aws-lc-rs or ring) to be available. Users enabling
the `rustls` feature should be informed of this requirement. Consider
switching to `reqwest/rustls-tls-native-roots` which bundles a default
provider, or documenting the requirement explicitly.
