# Shared audio output layer

This is the code behind OxiGuitar's four audio output plugins: [PipeWire](https://mateusrdp.github.io/oxiguitar-pipewire/), [ALSA](https://mateusrdp.github.io/oxiguitar-alsa/), [WASAPI](https://mateusrdp.github.io/oxiguitar-wasapi/) and [Core Audio](https://mateusrdp.github.io/oxiguitar-coreaudio/). It is a library for those plugins, not a plugin: nothing here is installed on its own.

It uploads nothing and downloads nothing. It only plays sound on your computer.

## What an audio output does

OxiGuitar works out the sound of a song as a stream of numbers, about 48,000 of them a second for each speaker. Something has to hand those numbers to the sound card, in small batches, always on time: if a batch is late, you hear a click or a gap. That is the job of the operating system's sound system, and of the output plugin that connects OxiGuitar to it.

Every system has its own sound system with its own way of being called. [cpal](https://github.com/RustAudio/cpal), a Rust audio library, already speaks all of them; it calls each one a *host*. This layer does the whole job on top of one of cpal's hosts:

- it lists the outputs (speakers, headphones, USB interfaces) by name;
- it opens a stream on the one the player picked, at 48,000 frames a second when the device takes that rate, else at the device's own;
- for every batch the device asks for, it asks OxiGuitar's core to fill it, and tells the core how long the batch takes to reach the speaker, so the cursor follows what you hear;
- it closes the stream when the player stops.

Each plugin then only says which host it uses: PipeWire's, ALSA's, WASAPI's or Core Audio's.

Two words come up when people talk about this:

- **Latency** is the delay between OxiGuitar sending sound and you hearing it.
- **The buffer** is how much sound is handed over at a time. A bigger buffer is safer against clicks on a busy computer, at the cost of more latency.

[How to use it](use-cases.md) shows the code of a plugin built on it.
