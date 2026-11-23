# Credits

Thank you to the people whose work this layer is built on.

## Code we use

| Project | Authors | What it does here | Licence |
| --- | --- | --- | --- |
| [OxiGuitar's plugin SDK](https://github.com/mateusrdp/oxiguitar) (`oxiguitar-plugin`) | Mateus Rocha de Paula and the OxiGuitar contributors | The plugin interface: the service, the host, the core's audio callback | MIT OR Apache-2.0 |
| [cpal](https://github.com/RustAudio/cpal) | The RustAudio contributors | Devices and the audio stream, on every system | Apache-2.0 |
| [alsa](https://github.com/diwic/alsa-rs) | David Henningsson and contributors | cpal's ALSA host, built on Linux | Apache-2.0 OR MIT |
| [Rust](https://www.rust-lang.org/) and its standard library | The Rust Project | Everything else | MIT OR Apache-2.0 |

cpal's other hosts bring their own bindings (PipeWire's, Windows', Apple's) into the plugins that turn them on; each plugin's own Credits page names them.

## Tools

| Tool | Use |
| --- | --- |
| [mdBook](https://github.com/rust-lang/mdBook) | Builds this manual |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | Checks the licences of everything we build with |

## Thanks

To the RustAudio community for cpal, which turns four very different sound systems into one.
