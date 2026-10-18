<p align="center"><img src="assets/volund-emblem.jpg" width="300" alt="Völund Audio emblem: a winged symbol in antique bronze"></p>

# Völund Audio

**Forge your sound.**

Völund Audio is a local desktop application for editing stems, mixing, mastering, preparing albums, and restoring vinyl recordings. The name comes from Völund, the master smith of Norse legend.

**Development preview: 0.95 · Windows, macOS, and Linux · Experimental VST3 support · English and Swedish interface · MIT license**

[Getting started](#getting-started) · [Features](#features) · [Workflow](#typical-workflow) · [English manual](docs/MANUAL.en.md) · [Svensk manual](docs/MANUAL.sv.md) · [Build and test](#build-and-test)

> The project is currently using **0.95** as its preview label. The Cargo and Tauri package metadata still reports **0.9.0**.

## Features

| Area | What you can do |
|---|---|
| **Stem editing** | Open Suno ZIP archives, WAV, MP3, or FLAC files. View tracks on a shared timeline, then cut, mute, fade, or pan selected regions. Undo and redo edits per track. |
| **Mixing** | Adjust track levels, mute, or solo tracks. Trim the combined stereo mix before mastering. |
| **Easy Podcast** | Work with one microphone WAV per voice on a shared timeline. Cut or mute the same section across the whole mix, add an optional opening and closing jingle, then continue to mastering. |
| **Mastering** | Use separate Spotify, Vinyl, CD, and Cassette profiles. Apply EQ, compression, an optional VST3 effect, limiting, and dithering. Compare before and after, and view LUFS and true-peak measurements. |
| **Albums** | Loudness-match finished tracks, make individual adjustments, and export tracks or side A/B track lists. |
| **Vinyl restoration** | Reduce clicks, crackle, and rumble. Detect possible track boundaries, adjust markers, and export split tracks. Identify marked tracks with AcoustID and fetch a matched release’s complete track list from MusicBrainz. Optionally separate a stereo mix into drums, bass, other, and vocals with Demucs-RS. |
| **Metadata and projects** | View, edit, and clear metadata. Save source audio and edits in a project that can be reopened later. |

### VST3 support

VST3 support is experimental in this preview. Völund scans the standard plugin folders at startup. A selected effect is loaded when needed and processed offline, before the final limiter and dither. Its editor opens in a separate native window.

VST3 effects are not used during real-time playback. Plugins without an editor window cannot be opened from Völund. Plugin settings are stored with the mastering profile in the project, and bypass is available for before/after comparisons. VST3 hosting is not available in the FreeBSD browser fallback.

### Local processing and Demucs

Audio processing runs locally. Demucs is an optional feature: the Windows test package includes the Demucs-RS engine, but not the model weights. When you start separation, the weights are downloaded from Hugging Face; your audio is not uploaded. The engine is currently bundled only with the Windows test package. No AI subscription is required.

Chromaprint fingerprints are created locally. If you start track identification, only the fingerprint and duration are sent to AcoustID; your audio is not uploaded. The feature requires internet access and an AcoustID application key. When you select a suggested release, Völund requests its full track listing from MusicBrainz using the release ID. This can help compare a rip of side A or B with the disc, but release pressings may differ and MusicBrainz may not encode vinyl-side positions. The matched recording is highlighted; Völund does not create or move markers automatically.

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

## Easy Podcast mode

Use **Easy Podcast** for a simple spoken-word workflow. Export one WAV per microphone from the same recording session. Keep the same start point and full recording length on every file, including silence at the beginning. Völund keeps these tracks on one timeline; this mode does not move individual tracks to correct different start times.

Use the shared mix waveform to select a passage, then cut or mute that passage across all voices. Cutting shortens the whole programme while keeping voices together. Muting leaves the timeline length unchanged. Each track also has simple mute and listen-to-this-voice controls.

Optional opening and closing jingles can be loaded as mono or stereo WAV files (up to 30 seconds and 12 MB each). They are stored in the <code>.vcleaner</code> project and included in preview, mastering, and exports. Save the project separately from exporting the finished episode.

## Saving and exporting

| Control | Result |
|---|---|
| **Export selected track…** / **Ctrl+Shift+E** | Choose WAV (32-bit float), FLAC (24-bit lossless), MP3 (320 kbps), or AAC in an M4A container (256 kbps). WAV retains the editable metadata fields. |
| **Save edited mix…** | A stereo mix before mastering. |
| **Save current profile as WAV…** / **Save selected versions as ZIP…** | Finished master files; the ZIP also contains a measurement report. |
| **Save project…** | Source audio, stem edits, mix trims and levels, mastering settings, optional podcast jingles, and saved VST3 settings in a `.vcleaner` project file. |
| **Save side project…** | The album's source tracks, order, sides, and level-matching settings in a `.vcleaner` project file. |

**Exporting audio does not save the project.** There is no automatic project saving or crash recovery. Edits do not overwrite the source files; choose a new filename when exporting. FLAC is lossless. MP3 and AAC are lossy encodings; converting an already-compressed source cannot restore lost detail.

## Manuals

- [English user manual](docs/MANUAL.en.md) — a complete guide to editing stems, mixing, mastering, album preparation, vinyl restoration, and project files.
- [Svensk användarmanual](docs/MANUAL.sv.md) — the Swedish version of the guide.

Both manuals include practical examples, troubleshooting, and technical limits.

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