<p align="center"><img src="assets/volund-emblem.jpg" width="300" alt="Völund Audio emblem: a winged symbol in antique bronze"></p>

# Völund Audio

**Forge your sound.**

Völund Audio is a local desktop application for editing stems, mixing, mastering, preparing albums, and restoring vinyl recordings. The name comes from Völund, the master smith of Norse legend.

**Development preview: 0.95 · Windows, macOS, and Linux · Experimental VST3 support · English and Swedish interface · MIT license**

[Getting started](#getting-started) · [Features](#features) · [Workflow](#typical-workflow) · [Manual](#manual) · [Build and test](#build-and-test)

> The project is currently using **0.95** as its preview label. The Cargo and Tauri package metadata still reports **0.9.0**.

## Features

| Area | What you can do |
|---|---|
| **Stem editing** | Open Suno ZIP archives, WAV, MP3, or FLAC files. View tracks on a shared timeline, then mute, fade, or pan selected regions. Undo and redo edits per track. |
| **Mixing** | Adjust track levels, mute, or solo tracks. Trim the combined stereo mix before mastering. |
| **Mastering** | Use separate Spotify, Vinyl, CD, and Cassette profiles. Apply EQ, compression, an optional VST3 effect, limiting, and dithering. Compare before and after, and view LUFS and true-peak measurements. |
| **Albums** | Loudness-match finished tracks, make individual adjustments, and export tracks or side A/B track lists. |
| **Vinyl restoration** | Reduce clicks, crackle, and rumble. Detect possible track boundaries, adjust markers, and export split tracks. Optionally separate a stereo mix into drums, bass, other, and vocals with Demucs-RS. |
| **Metadata and projects** | View, edit, and clear metadata. Save source audio and edits in a project that can be reopened later. |

### VST3 support

VST3 support is experimental in this preview. Völund scans the standard plugin folders at startup. A selected effect is loaded when needed and processed offline, before the final limiter and dither. Its editor opens in a separate native window.

VST3 effects are not used during real-time playback. Plugins without an editor window cannot be opened from Völund. Plugin settings are stored with the mastering profile in the project, and bypass is available for before/after comparisons. VST3 hosting is not available in the FreeBSD browser fallback.

### Local processing and Demucs

Audio processing runs locally. Demucs is an optional feature: the Windows test package includes the Demucs-RS engine, but not the model weights. When you start separation, the weights are downloaded from Hugging Face; your audio is not uploaded. The engine is currently bundled only with the Windows test package. No AI subscription is required.

Suggestions for unwanted sounds and mastering guidance are based on signal analysis, not generative AI.

## Downloads

Builds are published as GitHub Actions artifacts after a successful workflow run. Open [GitHub Actions](https://github.com/Morrberg/Volund-Audio/actions), choose a successful run, and download the artifact for your platform. Artifacts are temporary and currently retained for 30 days.

| Platform | Artifact | Notes |
|---|---|---|
| **Windows** | `Volund-Audio-Windows` | Contains `volund-audio.exe` and the optional Demucs-RS engine. Requires [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/). |
| **macOS** | `Volund-Audio-macOS` | Disk image (`.dmg`). |
| **Ubuntu Linux** | `Volund-Audio-Ubuntu` | Debian package (`.deb`) and AppImage. |

These are development/test builds, not signed release installers. Windows SmartScreen or macOS security checks may show a warning.

## Getting started

1. Download the artifact for your operating system from a successful [GitHub Actions run](https://github.com/Morrberg/Volund-Audio/actions).
2. Extract the Windows ZIP, open the macOS disk image, or install/run the Ubuntu package.
3. Start Völund Audio. On Windows, make sure WebView2 Runtime is installed.
4. Choose **Open stems / project** and open the stems for one song, such as a Suno ZIP archive.
5. Select a track, mark a region, and try an edit such as **Mute**. Choose **Save project…** if you want to continue later.

The Windows test build is unsigned, so SmartScreen may warn you. The [Swedish manual](docs/MANUAL.sv.md#starta) explains how to check the source and proceed with a build you trust.

## Typical workflow

1. **Edit stems:** mute an unwanted vocal sound in just that region; the other tracks stay in sync.
2. **Shape the mix:** balance track levels and pan as needed. Use **Mix · trim before mastering** to shorten an intro or outro across all master versions.
3. **Create masters:** adjust the mastering profiles, compare before and after, and export the versions you need.
4. **Prepare an album:** use the album section's file picker to add finished songs made with the same profile, match levels, and export tracks or side A/B.

The album section works with finished songs. Opening several songs as stems in the main view would place them on top of each other in one mix.

## Saving and exporting

| Control | Result |
|---|---|
| **Save selected track…** / **Ctrl+S** | A WAV file of the edited active track. |
| **Save edited mix…** | A stereo mix before mastering. |
| **Save current profile as WAV…** / **Save selected versions as ZIP…** | Finished master files; the ZIP also contains a measurement report. |
| **Save project…** | Source audio, stem edits, mix trims and levels, mastering settings, and saved VST3 settings in a `.vcleaner` project file. |
| **Save side project…** | The album's source tracks, order, sides, and level-matching settings in a `.vcleaner` project file. |

**Exporting audio does not save the project.** There is no automatic project saving or crash recovery. Edits do not overwrite the source files; choose a new filename when exporting.

## Manual

The [user manual](docs/MANUAL.sv.md) is currently in Swedish. It includes examples for:

- Muting an unwanted “Wrooow” without moving the stems out of sync.
- Panning drums from left to right.
- Trimming an intro in the mix before creating master versions.
- Exporting the same song for several formats.
- Matching album levels while keeping an intentionally quieter intro.

Profile defaults are starting points, not guarantees that a master meets a particular service's or manufacturer's requirements. Vinyl and cassette masters should be checked against the recipient's specifications. CD export produces WAV audio, not a complete DDP master. Album level matching adjusts loudness; it does not automatically match EQ or tone.

## Build and test

Install Rust and Node.js 24. Tauri also requires platform-specific desktop libraries; the Linux packages used by CI are listed in [.github/workflows/rust.yml](.github/workflows/rust.yml).

Run the tests and build with:

```sh
cargo test
node tests/editor.test.cjs
node tests/repair.test.cjs
cargo build --release
```

The workflow builds desktop packages for Windows, macOS, and Ubuntu Linux. FreeBSD currently uses the browser fallback; desktop and VST3 support have not been verified there.

## Project and license

Völund Audio was previously called **Vocal Cleaner**. Existing `.vcleaner` project files are still supported. The repository is [Morrberg/Volund-Audio](https://github.com/Morrberg/Volund-Audio).

This project is licensed under the [MIT License](LICENSE).