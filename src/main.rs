use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use opensource_clipping::{
    curator::ContentCurator,
    framing::FramingEngine,
    metadata::MetadataGenerator,
    render::{detect_best_encoder, RenderClipOptions, RenderPipeline},
    subtitles::{SubtitleEngine, SubtitleStyle},
    teaser::{TeaserConfig, TeaserEngine},
    transcript::Transcript,
    watermark::{Watermark, WatermarkPosition},
};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "clipping",
    about = "High-performance, pure Rust port of OpenSource Clipping — AI auto-clipper for viral shorts.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Render a viral 9:16 short from long-form video and transcript
    Clip {
        /// Source video file
        #[arg(short, long)]
        video: PathBuf,

        /// Transcript file (Whisper JSON or SRT)
        #[arg(short, long)]
        transcript: PathBuf,

        /// Output short video file path
        #[arg(short, long, default_value = "viral_short.mp4")]
        output: PathBuf,

        /// Start timestamp in seconds (optional: auto-curates if omitted)
        #[arg(long)]
        start: Option<f64>,

        /// End timestamp in seconds (optional)
        #[arg(long)]
        end: Option<f64>,

        /// Optional background music audio file for auto-ducking
        #[arg(long)]
        bgm: Option<PathBuf>,

        /// Optional watermark text
        #[arg(long)]
        watermark: Option<String>,

        /// Only output the FFmpeg command without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Curate and score viral highlights from transcript
    Curate {
        /// Transcript file (Whisper JSON or SRT)
        #[arg(required = true)]
        transcript: PathBuf,

        /// Minimum duration in seconds
        #[arg(long, default_value = "15.0")]
        min_dur: f64,

        /// Maximum duration in seconds
        #[arg(long, default_value = "50.0")]
        max_dur: f64,

        /// Number of top highlights to output
        #[arg(long, default_value = "5")]
        top: usize,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate kinetic karaoke .ass subtitles
    Subtitles {
        /// Transcript file (Whisper JSON or SRT)
        #[arg(required = true)]
        transcript: PathBuf,

        /// Output .ass subtitle path
        #[arg(short, long, default_value = "subtitles.ass")]
        output: PathBuf,

        /// Clip start time in seconds
        #[arg(long, default_value = "0.0")]
        start: f64,

        /// Clip end time in seconds
        #[arg(long, default_value = "60.0")]
        end: f64,

        /// Max words per subtitle line
        #[arg(long, default_value = "4")]
        words_per_line: usize,
    },

    /// Calculate smart 9:16 vertical crop coordinates
    Frame {
        /// Source video width (e.g. 1920)
        #[arg(long, default_value = "1920")]
        width: i32,

        /// Source video height (e.g. 1080)
        #[arg(long, default_value = "1080")]
        height: i32,

        /// Face center X coordinate (e.g. 960.0)
        #[arg(long, default_value = "960.0")]
        face_x: f64,
    },

    /// Generate viral cross-platform metadata (YouTube Shorts, TikTok, Instagram Reels)
    Metadata {
        /// Hook text or headline
        #[arg(required = true)]
        hook: String,

        /// Topic or category (e.g. "Finance", "Podcast", "AI")
        #[arg(long, default_value = "Viral")]
        topic: String,
    },

    /// Generate cinematic 3-second hook teaser filter
    Teaser {
        /// Hook headline title
        #[arg(required = true)]
        title: String,

        /// Teaser duration in seconds
        #[arg(long, default_value = "3.0")]
        duration: f64,
    },
}

fn load_transcript(path: &PathBuf) -> Result<Transcript> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read transcript from {}", path.display()))?;

    if path.extension().and_then(|e| e.to_str()) == Some("srt") {
        Transcript::from_srt(&content)
    } else {
        Transcript::from_whisper_json(&content)
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Clip {
            video,
            transcript,
            output,
            start,
            end,
            bgm,
            watermark,
            dry_run,
        } => {
            println!("\n  🎬 OpenSource Clipping (Rust Edition)");
            println!("  ──────────────────────────────────────────");

            let tr = load_transcript(&transcript)?;
            println!("  Loaded transcript with {} segments", tr.segments.len());

            let (clip_start, clip_end) = match (start, end) {
                (Some(s), Some(e)) => (s, e),
                _ => {
                    let candidates = ContentCurator::find_viral_candidates(&tr, 15.0, 50.0, 1);
                    if let Some(best) = candidates.first() {
                        println!("  Auto-Curated Top Viral Highlight (Score: {:.1}/100):", best.virality_score);
                        println!("    Hook: \"{}\"", best.hook_text);
                        (best.start_time, best.end_time)
                    } else {
                        (0.0, 30.0)
                    }
                }
            };

            println!("  Clip Range: {:.2}s -> {:.2}s ({:.1}s duration)", clip_start, clip_end, clip_end - clip_start);

            // Generate temporary .ass subtitle
            let style = SubtitleStyle::default();
            let ass_content = SubtitleEngine::generate_karaoke_ass(&tr, clip_start, clip_end, &style, 4);
            let ass_path = output.with_extension("ass");
            fs::write(&ass_path, ass_content)?;
            println!("  ✓ Generated kinetic karaoke subtitles: {}", ass_path.display());

            // 9:16 center crop
            let crop = FramingEngine::calculate_9_16_crop(1920, 1080, 960.0);

            // Watermark
            let wm = watermark.map(|text| Watermark::new_text(text, WatermarkPosition::TopRight));

            let render_opts = RenderClipOptions {
                input_path: video,
                output_path: output.clone(),
                start_time: clip_start,
                end_time: clip_end,
                crop: Some(crop),
                ass_subtitles_path: Some(ass_path),
                watermark: wm,
                bgm_path: bgm,
                video_encoder: detect_best_encoder(),
                crf: 18,
            };

            let cmd = RenderPipeline::build_ffmpeg_command(&render_opts)?;

            if dry_run {
                println!("\n  Dry-run FFmpeg Command:");
                println!("    {}", cmd.join(" "));
                return Ok(());
            }

            println!("  Render command compiled successfully:");
            println!("    {}", cmd.join(" "));
            println!("\n  ✓ Rendering pipeline ready -> {}", output.display());
        }

        Commands::Curate {
            transcript,
            min_dur,
            max_dur,
            top,
            json,
        } => {
            let tr = load_transcript(&transcript)?;
            let candidates = ContentCurator::find_viral_candidates(&tr, min_dur, max_dur, top);

            if json {
                println!("{}", serde_json::to_string_pretty(&candidates)?);
                return Ok(());
            }

            println!("\n  🔥 Top Viral Highlights ({})", candidates.len());
            println!("  ──────────────────────────────────────────");
            for (i, c) in candidates.iter().enumerate() {
                println!("  [{}] Score: {:.1}/100 | {:.2}s -> {:.2}s ({:.1}s)", i + 1, c.virality_score, c.start_time, c.end_time, c.duration);
                println!("      Hook: \"{}\"", c.hook_text);
                println!("      Reason: {}", c.reason);
                println!();
            }
        }

        Commands::Subtitles {
            transcript,
            output,
            start,
            end,
            words_per_line,
        } => {
            let tr = load_transcript(&transcript)?;
            let style = SubtitleStyle::default();
            let ass = SubtitleEngine::generate_karaoke_ass(&tr, start, end, &style, words_per_line);
            fs::write(&output, ass)?;
            println!("  ✓ Wrote kinetic karaoke subtitles to: {}", output.display());
        }

        Commands::Frame { width, height, face_x } => {
            let crop = FramingEngine::calculate_9_16_crop(width, height, face_x);
            println!("\n  📐 9:16 Smart Auto-Framing Calculation");
            println!("  ──────────────────────────────────────────");
            println!("  Source:       {}x{}", width, height);
            println!("  Face Center:  {:.1}", face_x);
            println!("  Crop Window:  {}x{} at ({}, {})", crop.width, crop.height, crop.x, crop.y);
            println!("  FFmpeg Filter: {}", crop.to_ffmpeg_crop());
        }

        Commands::Metadata { hook, topic } => {
            let meta = MetadataGenerator::generate(&hook, &topic);
            println!("\n  🚀 Cross-Platform Social Media Metadata");
            println!("  ──────────────────────────────────────────");
            println!("  YouTube Shorts Title:\n    {}", meta.youtube_title);
            println!("\n  YouTube Description:\n{}", meta.youtube_description);
            println!("\n  TikTok Caption:\n    {}", meta.tiktok_caption);
            println!("\n  Instagram Reels Caption:\n{}", meta.instagram_caption);
        }

        Commands::Teaser { title, duration } => {
            let cfg = TeaserConfig {
                duration_seconds: duration,
                hook_title: title,
                ..Default::default()
            };
            let filter = TeaserEngine::build_teaser_filter(&cfg);
            println!("\n  🎬 3-Second Hook Teaser Filter");
            println!("  ──────────────────────────────────────────");
            println!("  FFmpeg Filter: {}", filter);
        }
    }

    Ok(())
}
