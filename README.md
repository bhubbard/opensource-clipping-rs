# `opensource-clipping-rs` 🦀🎬

[![CI](https://github.com/bhubbard/opensource-clipping-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bhubbard/opensource-clipping-rs/actions/workflows/ci.yml)
[![Coverage](https://github.com/bhubbard/opensource-clipping-rs/actions/workflows/coverage.yml/badge.svg)](https://github.com/bhubbard/opensource-clipping-rs/actions/workflows/coverage.yml)
[![Pages](https://github.com/bhubbard/opensource-clipping-rs/actions/workflows/pages.yml/badge.svg)](https://code.brandonhubbard.com/opensource-clipping-rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance, native pure **Rust** port of [NaufalRizqullah/opensource-clipping](https://github.com/NaufalRizqullah/opensource-clipping) integrated with **PySceneDetect (`scenedetect-rs`)** 64-bin HSV/RGB shot boundary detection and **Netflix MatchCut (`matchcut-rs`)** cosine continuity scoring — turning long-form videos & podcasts into high-retention 9:16 viral shorts with EMA smart face-tracking, kinetic `.ass` karaoke subtitles, and EBU R128 sidechain audio ducking.

> **Interactive Studio & Live Viral Shorts Workbench**: [**https://code.brandonhubbard.com/opensource-clipping-rs/**](https://code.brandonhubbard.com/opensource-clipping-rs/)  
> **Empirical Benchmarks**: [**BENCHMARKS.md**](BENCHMARKS.md) (**54.8x faster** than Python `opensource-clipping` + `PySceneDetect`, **3.6 MB** peak RSS)

---

## ⚡ Highlights & Architecture

- **🎬 650+ FPS Shot Boundary & MatchCut Engine (`src/scenedetect.rs`)**:
  - **Content-Aware Cut Detector (`detect_content_cuts`)**: 64-bin quantized RGB/HSV histogram Manhattan ($L_1$) delta detector matching `Breakthrough/PySceneDetect`.
  - **Adaptive Rolling-Window Detector (`detect_adaptive_cuts`)**: Dynamic rolling-window ratio filter eliminating false triggers during camera pans and strobe flashes.
  - **Threshold Luma Fade Detector (`detect_luma_fades`)**: BT.709 luminance threshold crossing detector for fade-in and fade-out transitions.
  - **MatchCut Visual Continuity (`cosine_similarity`)**: Scores visual frame similarity across cuts for seamless jump-cut assembly.
- **🔥 AI Content Curation & Virality Scorer (`src/curator.rs`)**:
  - Sliding-window highlight extractor scoring question hooks, curiosity triggers, power words, and speech velocity (`140–190 WPM`).
- **📐 Smart 9:16 Auto-Framing & EMA Face Smoothing (`src/framing.rs`)**:
  - Converts `1920×1080` widescreen into `608×1080` (`9:16`) vertical crops with Exponential Moving Average (`FaceTrackerSmoother`) jitter suppression and dual-speaker podcast split-screen (`vstack=inputs=2`).
- **✨ Kinetic Karaoke Subtitles Engine (`src/subtitles.rs`)**:
  - Generates Advanced SubStation Alpha (`.ass` v4.00+) scripts with centisecond `{\k...}` word timing and `{\t(0,70,\fscx110\fscy110)}` kinetic pop animations.
- **🎵 Sidechain Audio Ducking & `-16.0 LUFS` Loudness (`src/audio_ducking.rs`)**:
  - Compiles FFmpeg `sidechaincompress` + `loudnorm=I=-16.0:TP=-1.5:LRA=11` broadcast audio filtergraphs.

---

## 📊 Performance Summary

| Pipeline Stage | Python Reference | Rust `opensource-clipping-rs` | Speedup |
| :--- | ---: | ---: | ---: |
| **Whisper JSON & SRT Transcript Ingest (1h)** | `38.40 ms` | **`0.62 ms`** | **61.9x** |
| **Sliding-Window Virality Hook Curation** | `19.20 ms` | **`0.31 ms`** | **61.9x** |
| **64-Bin RGB/HSV Shot Boundary Detection (1,800f)** | `142.00 ms` | **`2.75 ms`** | **51.6x** |
| **Kinetic `.ass` Karaoke Subtitle Generation** | `14.50 ms` | **`0.24 ms`** | **60.4x** |
| **Total End-to-End Host Pipeline** | **`220.90 ms`** | **`4.03 ms`** | **54.8x** |

---

## 📜 License

Licensed under the [MIT License](LICENSE).
