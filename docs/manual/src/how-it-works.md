# Architecture and strategy

`CpalDevice` is the `audio-device` service of the plugin interface on one of cpal's hosts. The whole layer is one file, `src/lib.rs`.

- **One thread for the streams.** A cpal stream may have to stay on the thread that made it, so the streams live on a thread of their own, started on first use. It sleeps until the player asks it to open or close a stream; nothing polls.
- **The callback waits for nothing.** When the device wants a batch, its callback hands the core the buffer (`AudioOut::render`) with the delay to the speaker that cpal reports. It allocates nothing and takes no lock, because a late callback is a click.
- **The rate.** 48 kHz when the device takes it (the rate OxiGuitar's amp models are made for), else the device's own, which the player then resamples to.
- **The samples.** f32 when the device takes them. Devices that take only 16- or 32-bit integers get the f32 samples clamped to full scale and converted through a buffer made when the stream opens.
- **Stopping.** Closing a stream drops it, which stops it: its callback isn't called again once `close` returns.

The strategy is one plugin for each system's own sound API, so each can be installed, updated and left out on its own, and one layer for what they have in common, so it is written and fixed once.

## Current limitations

- **Delay as cpal reports it.** The cursor follows the delay each host reports for a batch. A host that reports none gets a cursor a buffer early.
- **Sample formats.** Only f32, 16-bit and 32-bit integer outputs are taken; a device that offers nothing else is refused with a message saying so.
- **No buffer size of our own.** The device's default buffer is used.
- **Your system's sound must already work.** The layer doesn't repair or configure the sound system.

## Left out on purpose

- **A sound library of our own.** cpal already speaks every system's sound API, so this layer stays small.
- **Living in OxiGuitar's repository.** It did until OxiGuitar 0.21.6, but the core itself never used it: being in the core's workspace only made every core build compile cpal and need ALSA's headers.
- **Recording.** The outputs only play. OxiGuitar has no audio input.

## Where you can help

- **Testing on real hardware**, especially devices that take only integers, USB interfaces and Bluetooth headphones.
- **A chosen buffer size**, for those who want lower latency.

Every change needs its tests and a row on the [Features](features.md) page; the [Testing](testing.md) page says how to run them.
