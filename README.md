# oxiguitar-audio-cpal

The code that [OxiGuitar](https://github.com/mateusrdp/oxiguitar)'s four audio output plugins share: [oxiguitar-pipewire](https://github.com/mateusrdp/oxiguitar-pipewire), [oxiguitar-alsa](https://github.com/mateusrdp/oxiguitar-alsa), [oxiguitar-wasapi](https://github.com/mateusrdp/oxiguitar-wasapi) and [oxiguitar-coreaudio](https://github.com/mateusrdp/oxiguitar-coreaudio). OxiGuitar is a free, open-source guitar tablature and score viewer, editor and player for Windows, macOS and Linux.

In plain words: OxiGuitar works out the sound, and an output plugin hands it to the operating system's sound system on time, batch after batch. The four outputs do the same job on four sound systems, so the job itself lives here once, on top of [cpal](https://github.com/RustAudio/cpal), and each plugin only says which of cpal's sound systems it uses. The [manual](https://mateusrdp.github.io/oxiguitar-audio-cpal/) explains how.

- A library, not a plugin: there is no `.oxp` file and nothing to install. The plugins depend on it by git tag.
- It plays sound only on your machine: it uploads and downloads nothing.

## Use it

```toml
[dependencies]
oxiguitar-plugin = { git = "https://github.com/mateusrdp/oxiguitar", tag = "v0.11.0" }
oxiguitar-audio-cpal = { git = "https://github.com/mateusrdp/oxiguitar-audio-cpal", tag = "v0.1.0" }
cpal = { version = "0.18.2", default-features = false }
```

Use the same `oxiguitar-plugin` tag as this repository's `Cargo.toml`: the service it gives you must be the SDK's own type. The manual's [How to use it](https://mateusrdp.github.io/oxiguitar-audio-cpal/use-cases.html) page has the code.

## Build and test

Rust 1.99 or later from [rustup](https://rustup.rs); on Debian and Ubuntu `libasound2-dev` (`alsa-lib-devel` on Fedora), because cpal always builds its ALSA host on Linux.

```sh
cargo test                     # the tests; no sound card needed
python3 scripts/test-all.py    # every test, as the manual's Testing page says
```

## History

This crate lived in the OxiGuitar repository (`crates/oxiguitar-audio-cpal`) until OxiGuitar 0.21.6; its history is there. It moved because the core itself never used it.

## Credits

Built on [cpal](https://github.com/RustAudio/cpal) (Apache-2.0) and OxiGuitar's plugin SDK. Thanks to the RustAudio community. The manual's [Credits](https://mateusrdp.github.io/oxiguitar-audio-cpal/credits.html) page lists every project with its authors and licence. Thank you to all of them.

## Licence

GPL-3.0-or-later. See LICENSE. Contributions are welcome under the same terms; see [CONTRIBUTING.md](CONTRIBUTING.md).
