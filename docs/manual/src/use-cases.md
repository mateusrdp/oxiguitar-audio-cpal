# How to use it

An output plugin built on this layer is a few lines: the plugin offers one `audio-device` service, and that service is a `CpalDevice` on its host.

1. Depend on the SDK and on this layer, at the tags its own `Cargo.toml` uses:

   ```toml
   [dependencies]
   oxiguitar-plugin = { git = "https://github.com/mateusrdp/oxiguitar", tag = "v0.11.0" }
   oxiguitar-audio-cpal = { git = "https://github.com/mateusrdp/oxiguitar-audio-cpal", tag = "v0.1.0" }
   # Turn on your host's cpal feature here, if it has one ("pipewire").
   cpal = { version = "0.18.2", default-features = false }
   ```

   The SDK's tag must be the one this layer uses: the service it gives you is the SDK's own type, and two tags make two types that don't match.

2. Make the service when the plugin starts, on your host:

   ```rust
   use oxiguitar_audio_cpal::CpalDevice;

   let device = CpalDevice::new(host, cpal::HostId::Alsa, "ALSA");
   ```

   The name is the one messages use ("ALSA has no output").

3. Offer it as the `audio-device` service your manifest names, and nothing else. [oxiguitar-alsa](https://github.com/mateusrdp/oxiguitar-alsa)'s `src/lib.rs` is the whole of a plugin built this way.

The player then calls the service's methods (`devices`, `open`, `close`) through the plugin interface; the plugin developer guide in OxiGuitar's repository describes them.
