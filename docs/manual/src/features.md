# Features and their tests

Everything this layer does, and the tests that check it. The test names are the ones `cargo test` prints. `scripts/test-all.py` fails when a feature here names no test, or names a test that doesn't exist, so a new feature gets its row and its tests in the same pull request (the [Testing](testing.md) page says how to run them).

Hearing the sound needs a sound card, which CI doesn't have, so the tests check everything up to it, on the host every system has (ALSA on Linux, WASAPI on Windows, Core Audio on macOS). Each output plugin's contract suite checks the layer again inside its plugin file.

| Feature | What it does | Tests |
| --- | --- | --- |
| Says when it can't play | Probing tells the player whether there's an output, and why not when there isn't. | `tests::probing_says_whether_there_is_an_output` |
| Lists the outputs | The system's output devices by name, for the footer's output chooser. | `tests::listing_the_outputs_answers_with_or_without_a_device` |
| Refuses wrong requests | A damaged request to open or close an output, or a method it doesn't have, gets an error, never a crash. | `tests::wrong_requests_are_refused` |
| Opens and closes streams | Opening needs OxiGuitar's core to make the sound and says why when there is none; closing a stream that isn't open is harmless. | `tests::opening_without_a_core_says_why`, `tests::closing_an_unknown_stream_is_harmless` |
| Integer devices | Devices that take only 16- or 32-bit samples get them clamped to full scale, so a loud note doesn't wrap round into a click. | `tests::samples_for_integer_devices_are_clamped_and_scaled` |
