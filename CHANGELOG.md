# Changelog

All notable changes to this repository are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.1] - 2026-11-25

### Changed

- The SDK at the tag `v0.23.0` instead of the commit before it (the same SDK), so the four audio-device plugins can name it by tag too.

## [0.2.0] - 2026-11-25

### Changed

- **The SDK at mateusrdp/oxiguitar#348's commit** (plugin files are tar archives, mateusrdp/oxiguitar#347), in step with the four audio-device plugins, which move with it; `scripts/test-all.py` 1.1.0, as in every repository.
- **CI outlasts a slow Ubuntu mirror**: it installs packages with `scripts/apt-install.sh`, the same file as in every OxiGuitar repository. apt is stopped only when its downloads have not grown for 2 minutes, then tried again on the next mirror; while they grow, it may take as long as it needs (24 Nov 2026: fixed 5-minute tries failed jobs on mirrors at 80 to 185 kB/s).

## [0.1.1] - 2026-11-23

### Changed

- Documented what a plugin's repository must do to depend on this layer: the same SDK tag and cpal version, `allow-git` in `deny.toml`, refreshing `Cargo.lock`, Dependabot, and the order of an SDK bump.

## [0.1.0] - 2026-11-23

### Added

- The shared audio output layer, moved from OxiGuitar's repository (`crates/oxiguitar-audio-cpal`, unchanged since OxiGuitar 0.11.0), on cpal 0.18.2 and the SDK's `v0.11.0` tag.
- Tests that need no sound card: probing, the device list, refused requests, closing, opening without a core, and the 16- and 32-bit sample conversions.
- A manual, the test runner (`scripts/test-all.py`) and its Features page.
