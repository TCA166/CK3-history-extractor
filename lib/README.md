# ck3_history_extractor_lib

[![Crates.io Version](https://img.shields.io/crates/v/ck3_history_extractor_lib)](https://crates.io/crates/ck3_history_extractor_lib)
[![docs.rs](https://img.shields.io/docsrs/ck3_history_extractor_lib)](https://docs.rs/ck3_history_extractor_lib)
[![License](https://img.shields.io/crates/l/ck3_history_extractor_lib)](../license.txt)

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
- `tokens`: Enables token loading; necessary for ironman parsing
- `serde`: Enables serde serialization
- `display`: Enables visualisation

### Tokens

Using the `tokens` feature requires some work.
CK3 ironman save files are serialized into a binary format, with strings
being encoded as indices in a token table. This token table is static per a
given game version. The easiest way to obtain it, is to get it via a debug
command.

1. Launch the game in debug mode (`-debug_mode` in launch options)
2. Open the console and use the `oos_dump` command
3. In `Paradox Interactive/Crusader Kings III` you will find a new folder called
   `oos`
4. The tokens are dumped in `oos/dummy/tokens_1.tok`

This library expects to find that file in the directory pointed to by the
`TOKENS_DIR` environment variable, or in the current working directory if that
variable is not set.

Many thanks to [jzebedee](https://github.com/jzebedee) for details on how to
obtain the tokens.

### Display

The visualisation consists of facilitating the rendering of HTML pages for each
save file entity. Each entity that can be visualised in such a way, will
also try to render necessary static assets, so for example, a title will try to
render its map. Since this is all a rather narrow feature scope, most users
will not need this feature.
