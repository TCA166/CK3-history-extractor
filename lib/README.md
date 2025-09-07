# ck3_history_extractor_lib

[![Crates.io Version](https://img.shields.io/crates/v/ck3_history_extractor_lib)](https://crates.io/crates/ck3_history_extractor_lib)
[![docs.rs](https://img.shields.io/docsrs/ck3_history_extractor_lib)](https://docs.rs/ck3_history_extractor_lib)
[![License](https://img.shields.io/crates/l/ck3_history_extractor_lib)](../LICENSE)

The core toolkit
of [`ck3_history_extractor`](https://github.com/TCA166/CK3-history-extractor).
If you want to make your own tool that uses ck3 save files, this library is for
you.

## Functionality

- Extraction of all core game entities like Titles, Characters, Dynasties, and more.
- Ironman support
- Localization resolving
- Serde serialization (optional)
- minijinja based entity visualisation (optional)

## Library features

- `permissive`: Supresses localization errors
- `serde`: Enables serde serialization
- `display`: Enables visualisation

### Permissive

For debugging purposes, the localization errors will be raised, unless
the `permissive` feature is enabled. This is probably not what you want in a
production environment.

### Display

The visualisation consists of facilitating the rendering of HTML pages for each
save file entity. Each entity that can be visualised in such a way, will
also try to render necessary static assets, so for example, a title will try to
render its map. Since this is all a rather narrow feature scope, most users
will not need this feature.
