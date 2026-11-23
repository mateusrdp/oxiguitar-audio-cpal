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

## What a plugin's repository must do

Depending on this layer asks more of the plugin's repository than a crate from crates.io would. Each point below was a build or CI failure when it was missed.

- **One SDK tag.** `oxiguitar-plugin` must be at the tag this layer's own `Cargo.toml` names. A second tag makes a second `Host` and `Service` type, and the build fails with types that look the same and do not match. Cargo does not warn when the tags differ in a lock file.
- **One cpal version, and the host's feature on your own dependency.** Use the same cpal version as this layer (0.18.2) with `default-features = false`, and turn on only your host's feature (`pipewire` for PipeWire; ALSA, WASAPI and Core Audio need none). This layer turns on no host feature of its own.
- **Linux builds need `libasound2-dev`** (`alsa-lib-devel` on Fedora) whatever the host, because cpal always builds its ALSA host there. The PipeWire host adds `libpipewire-0.3-dev` and `libclang-dev`. Those belong to the plugin's build, not to this layer's.
- **`deny.toml` must allow the source.** With `unknown-git = "deny"`, cargo-deny rejects this layer until `allow-git` lists `https://github.com/mateusrdp/oxiguitar-audio-cpal` next to `https://github.com/mateusrdp/oxiguitar`.
- **A new tag is not picked up by itself.** Change the tag in `Cargo.toml`, then run `cargo update -p oxiguitar-audio-cpal -p cpal` and commit `Cargo.lock`.
- **Dependabot must leave both tags alone.** Ignore `oxiguitar-*` in the plugin's `dependabot.yml`; the tags move together, by hand.
- **Nothing goes on crates.io.** This layer's release is its `v*` tag: there is no GitHub Release page and no packages to download.

### When the SDK tag moves

Do it in this order, so no plugin ever sees two tags:

1. In this repository, change the SDK tag, run the tests, merge, and release with oxiguitar-packages' Release button.
2. In each of the four plugins, change the SDK tag and this layer's tag to the new ones in the same pull request, and refresh `Cargo.lock` as above.
