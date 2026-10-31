//! # OpenSource Clipping (Rust Edition) 🦀🎬
//!
//! A high-performance, native pure Rust port of [NaufalRizqullah/opensource-clipping](https://github.com/NaufalRizqullah/opensource-clipping) —
//! transforms long-form videos & podcasts into viral shorts with smart face-tracking,
//! kinetic karaoke subtitles, audio ducking, and auto-hook curation.

pub mod audio_ducking;
pub mod curator;
pub mod framing;
pub mod metadata;
pub mod render;
pub mod scenedetect;
pub mod story;
pub mod subtitles;
pub mod teaser;
pub mod transcript;
pub mod watermark;

pub use audio_ducking::{AudioDuckingConfig, AudioDuckingEngine};
pub use curator::{ContentCurator, ViralCandidate};
pub use framing::{BoundingBox, CropWindow, FaceTrackerSmoother, FramingEngine};
pub use metadata::{MetadataGenerator, SocialMetadata};
pub use render::{detect_best_encoder, RenderClipOptions, RenderPipeline};
pub use scenedetect::{FrameHistogram, SceneBoundary, SceneDetector};
pub use story::{StoryClipPlan, StoryEngine};
pub use subtitles::{SubtitleEngine, SubtitleStyle};
pub use teaser::{TeaserConfig, TeaserEngine};
pub use transcript::{Transcript, TranscriptSegment, WordTiming};
pub use watermark::{Watermark, WatermarkKind, WatermarkPosition};
