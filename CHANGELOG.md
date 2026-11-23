# Changelog

All notable changes to this repository are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- Documented what a plugin's repository must do to depend on this layer: the same SDK tag and cpal version, `allow-git` in `deny.toml`, refreshing `Cargo.lock`, Dependabot, and the order of an SDK bump.

## [0.1.0] - 2026-11-23

### Added

- The shared audio output layer, moved from OxiGuitar's repository (`crates/oxiguitar-audio-cpal`, unchanged since OxiGuitar 0.11.0), on cpal 0.18.2 and the SDK's `v0.11.0` tag.
- Tests that need no sound card: probing, the device list, refused requests, closing, opening without a core, and the 16- and 32-bit sample conversions.
- A manual, the test runner (`scripts/test-all.py`) and its Features page.
