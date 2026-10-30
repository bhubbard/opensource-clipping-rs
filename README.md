# `opensource-clipping-rs` 🦀🎬

[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-16%20Passing-brightgreen.svg)]()

A high-performance, native pure **Rust** port of [NaufalRizqullah/opensource-clipping](https://github.com/NaufalRizqullah/opensource-clipping) — an AI auto-clipper turning long-form videos & podcasts into high-retention viral shorts with smart face-tracking, kinetic karaoke subtitles (`.ass`), audio sidechain ducking, multi-hook teasers, and cross-platform social media metadata generation.

---

## ⚡ Highlights & Architecture

- **🚀 Native Compiled Speed**: Single static binary with zero Python dependencies, minimal memory footprint, and sub-second execution overhead.
- **🎙️ Word-Level Transcript Parsing**: Ingests Whisper JSON (faster-whisper, whisper.cpp, OpenAI) and SubRip (`.srt`) with centisecond-precision timing.
- **🔥 AI Content Curation & Virality Engine**:
  - Heuristic virality scoring based on question hooks, curiosity triggers, power words, and speech velocity (140-190 WPM).
  - Sliding-window candidate highlight extractor.
  - Formatter & parser for structured Google Gemini / OpenAI JSON curation prompts.
- **📐 Smart 9:16 Auto-Framing & Face Smoothing**:
  - Centered crop calculations from 16:9 widescreen to 9:16 vertical shorts.
  - Exponential Moving Average (EMA) face position smoothing to prevent camera shake/jitter.
  - Multi-speaker split-screen stacked layout compiler for podcasts.
- **✨ Kinetic Karaoke Subtitles Engine**:
  - Generates `.ass` (Advanced SubStation Alpha) scripts with word-by-word `{\k...}` timing.
  - Pop-in kinetic scaling animations, customizable fonts (Montserrat, The Bold Font), drop shadows, and high-visibility outlines.
- **🎵 Sidechain Audio Ducking & Loudness Normalization**:
  - Dynamic audio sidechain compressor filtergraph lowering background music (BGM) when speech is active.
  - Broadcast EBU R128 (`-16.0 LUFS`) loudness normalization for TikTok, YouTube Shorts, and Instagram Reels.
- **🎯 3-Second Hook Teaser**:
  - Cinematic zoom-in, dark vignette, and animated countdown headline overlays.
- **🏷️ Cross-Platform Metadata**:
  - Generates optimized YouTube Shorts titles & tags, TikTok captions with trending hashtags, and Instagram Reels captions.

---

## 🚀 Installation & Build

```bash
git clone https://github.com/bhubbard/opensource-clipping-rs.git
cd opensource-clipping-rs
cargo build --release
```

Binary will be available at `./target/release/clipping`.

---

## 💻 CLI Usage

### 1. Curate Top Viral Highlights
```bash
# Scan a transcript and find the top 5 viral moments
clipping curate podcast.srt --top 5

# Output as JSON
clipping curate transcript.json --top 3 --json
```

### 2. Generate Kinetic Karaoke Subtitles
```bash
# Generate high-retention .ass karaoke subtitles for a 30s highlight
clipping subtitles transcript.json --start 15.0 --end 45.0 --output subs.ass
```

### 3. Calculate 9:16 Smart Auto-Framing
```bash
# Calculate 9:16 crop window centered on face coordinate
clipping frame --width 1920 --height 1080 --face-x 960.0
```

### 4. Full Clip Production Pipeline
```bash
# Render complete vertical short with auto-curated highlight, subtitles, and watermark
clipping clip \
  --video podcast.mp4 \
  --transcript podcast.json \
  --watermark "@calljacob" \
  --output viral_clip.mp4 \
  --dry-run
```

### 5. Generate Cross-Platform Metadata
```bash
clipping metadata "Did you know the secret mistake everyone makes with money?" --topic "Finance"
```

---

## 🧪 Upstream Parity & Test Suite

Includes 16 automated integration and unit tests covering:
- Centisecond word-timing conversions & SRT transcript parser
- Virality scoring heuristics & sliding-window highlight extraction
- 9:16 crop boundaries, clamping, and EMA face smoothing
- Kinetic karaoke `.ass` formatting and line-wrapping
- Audio sidechain ducking and EBU R128 loudness filtergraphs
- Hardware-accelerated FFmpeg command compilation
- Cross-platform metadata generation

Run test suite:
```bash
cargo test
```

---

## 📜 License

Licensed under the [MIT License](LICENSE). Based on the concept and project by [Naufal Rizqullah](https://github.com/NaufalRizqullah/opensource-clipping).
