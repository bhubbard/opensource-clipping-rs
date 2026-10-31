use crate::framing::CropWindow;
use crate::watermark::Watermark;
use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RenderClipOptions {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub start_time: f64,
    pub end_time: f64,
    pub crop: Option<CropWindow>,
    pub ass_subtitles_path: Option<PathBuf>,
    pub watermark: Option<Watermark>,
    pub bgm_path: Option<PathBuf>,
    pub video_encoder: String,
    pub crf: u32,
}

impl Default for RenderClipOptions {
    fn default() -> Self {
        Self {
            input_path: PathBuf::from("input.mp4"),
            output_path: PathBuf::from("short.mp4"),
            start_time: 0.0,
            end_time: 30.0,
            crop: None,
            ass_subtitles_path: None,
            watermark: None,
            bgm_path: None,
            video_encoder: detect_best_encoder(),
            crf: 18,
        }
    }
}

/// Detects available hardware-accelerated video encoder based on platform
pub fn detect_best_encoder() -> String {
    if cfg!(target_os = "macos") {
        "h264_videotoolbox".to_string()
    } else {
        "libx264".to_string()
    }
}

pub struct RenderPipeline;

impl RenderPipeline {
    /// Builds the complete FFmpeg command to render the vertical short clip
    pub fn build_ffmpeg_command(opts: &RenderClipOptions) -> Result<Vec<String>> {
        let mut cmd = vec!["ffmpeg".to_string(), "-y".to_string()];

        // Trimming with accurate seek
        cmd.push("-ss".to_string());
        cmd.push(format!("{:.3}", opts.start_time));
        cmd.push("-to".to_string());
        cmd.push(format!("{:.3}", opts.end_time));

        cmd.push("-i".to_string());
        cmd.push(opts.input_path.to_string_lossy().to_string());

        // Optional BGM input
        if let Some(ref bgm) = opts.bgm_path {
            cmd.push("-stream_loop".to_string());
            cmd.push("-1".to_string());
            cmd.push("-i".to_string());
            cmd.push(bgm.to_string_lossy().to_string());
        }

        // Build video filter chain
        let mut video_filters = Vec::new();

        // 1. Crop to 9:16
        if let Some(ref crop) = opts.crop {
            video_filters.push(crop.to_ffmpeg_crop());
        }

        // 2. Scale to full 1080x1920
        video_filters.push("scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920".to_string());

        // 3. Subtitles
        if let Some(ref ass) = opts.ass_subtitles_path {
            let ass_escaped = ass.to_string_lossy().replace('\\', "/").replace(':', "\\:");
            video_filters.push(format!("subtitles='{}'", ass_escaped));
        }

        // 4. Watermark
        if let Some(ref wm) = opts.watermark {
            video_filters.push(wm.to_ffmpeg_filter());
        }

        cmd.push("-vf".to_string());
        cmd.push(video_filters.join(","));

        // Video codec & flags
        cmd.push("-c:v".to_string());
        cmd.push(opts.video_encoder.clone());

        if opts.video_encoder == "libx264" {
            cmd.push("-preset".to_string());
            cmd.push("fast".to_string());
            cmd.push("-crf".to_string());
            cmd.push(opts.crf.to_string());
        } else if opts.video_encoder == "h264_videotoolbox" {
            cmd.push("-b:v".to_string());
            cmd.push("8500k".to_string());
        }

        cmd.push("-c:a".to_string());
        cmd.push("aac".to_string());
        cmd.push("-b:a".to_string());
        cmd.push("192k".to_string());

        // Audio filters (ducking or normalization)
        if opts.bgm_path.is_some() {
            cmd.push("-filter_complex".to_string());
            cmd.push("[1:a]volume=0.20[bgm];[0:a][bgm]amix=inputs=2:duration=first[outa]".to_string());
            cmd.push("-map".to_string());
            cmd.push("0:v".to_string());
            cmd.push("-map".to_string());
            cmd.push("[outa]".to_string());
        } else {
            cmd.push("-af".to_string());
            cmd.push("loudnorm=I=-16:TP=-1.5:LRA=11".to_string());
        }

        cmd.push("-movflags".to_string());
        cmd.push("+faststart".to_string());

        cmd.push(opts.output_path.to_string_lossy().to_string());

        Ok(cmd)
    }

    /// Execute FFmpeg command
    pub fn execute(cmd: &[String]) -> Result<()> {
        if cmd.is_empty() {
            bail!("Empty FFmpeg command");
        }

        let status = Command::new(&cmd[0])
            .args(&cmd[1..])
            .status()?;

        if !status.success() {
            bail!("FFmpeg exited with error status: {}", status);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ffmpeg_command() {
        let opts = RenderClipOptions {
            crop: Some(CropWindow {
                x: 656,
                y: 0,
                width: 608,
                height: 1080,
            }),
            ass_subtitles_path: Some(PathBuf::from("subs.ass")),
            ..RenderClipOptions::default()
        };

        let cmd = RenderPipeline::build_ffmpeg_command(&opts).unwrap();
        assert_eq!(cmd[0], "ffmpeg");
        assert!(cmd.iter().any(|c| c.contains("crop=608:1080:656:0")));
        assert!(cmd.iter().any(|c| c.contains("subtitles='subs.ass'")));
        assert!(cmd.iter().any(|c| c == "+faststart"));
    }
}
