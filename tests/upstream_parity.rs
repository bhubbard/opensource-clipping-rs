use opensource_clipping::{
    audio_ducking::{AudioDuckingConfig, AudioDuckingEngine},
    curator::{ContentCurator, ViralCandidate},
    framing::{CropWindow, FaceTrackerSmoother, FramingEngine},
    metadata::MetadataGenerator,
    render::{RenderClipOptions, RenderPipeline},
    scenedetect::{FrameHistogram, SceneDetector},
    story::StoryEngine,
    subtitles::{SubtitleEngine, SubtitleStyle},
    teaser::{TeaserConfig, TeaserEngine},
    transcript::{Transcript, TranscriptSegment, WordTiming},
    watermark::{Watermark, WatermarkPosition},
};
use std::path::PathBuf;

#[test]
fn test_upstream_whisper_json_ingest() {
    let json_data = r#"{
        "language": "en",
        "segments": [
            {
                "id": 0,
                "start": 0.5,
                "end": 3.8,
                "text": "Why do 99% of people fail?",
                "words": [
                    { "word": "Why", "start": 0.5, "end": 0.9, "probability": 0.98 },
                    { "word": "do", "start": 0.9, "end": 1.2, "probability": 0.99 },
                    { "word": "99%", "start": 1.2, "end": 2.1, "probability": 0.95 },
                    { "word": "of", "start": 2.1, "end": 2.3, "probability": 0.99 },
                    { "word": "people", "start": 2.3, "end": 2.8, "probability": 0.97 },
                    { "word": "fail?", "start": 2.8, "end": 3.8, "probability": 0.99 }
                ],
                "speaker": "SPEAKER_00"
            }
        ]
    }"#;

    let tr = Transcript::from_whisper_json(json_data).unwrap();
    assert_eq!(tr.language.as_deref(), Some("en"));
    assert_eq!(tr.segments.len(), 1);
    assert_eq!(tr.total_words(), 6);
    assert_eq!(tr.segments[0].speaker.as_deref(), Some("SPEAKER_00"));
    assert_eq!(tr.segments[0].words[0].word, "Why");
    assert_eq!(tr.segments[0].words[0].duration_centiseconds(), 40);
}

#[test]
fn test_upstream_srt_ingest_and_time_slicing() {
    let srt = "1\n00:00:00,000 --> 00:00:05,000\nIntro hook\n\n2\n00:00:15,000 --> 00:00:30,000\nThe secret to success is momentum.\n\n3\n00:00:45,000 --> 00:00:55,000\nEnding clip.";
    let tr = Transcript::from_srt(srt).unwrap();
    assert_eq!(tr.segments.len(), 3);

    // Slice only the middle climax
    let sliced = tr.slice_by_time(10.0, 35.0);
    assert_eq!(sliced.segments.len(), 1);
    assert_eq!(sliced.segments[0].text, "The secret to success is momentum.");
    assert_eq!(sliced.segments[0].start, 15.0);
    assert_eq!(sliced.segments[0].end, 30.0);
}

#[test]
fn test_upstream_virality_scoring_heuristics() {
    // 1. High virality segment: strong question + curiosity power words + optimal pacing
    let viral_hook = "Did you know the secret mistake that ruined this millionaire?";
    let (score_high, reasons) = ContentCurator::score_segment(viral_hook, 20.0);
    assert!(score_high >= 70.0);
    assert!(reasons.contains("Strong question hook"));
    assert!(reasons.contains("curiosity triggers"));

    // 2. Low virality segment: mundane statement + slow pacing
    let boring = "Today we are looking at the report from last Tuesday.";
    let (score_low, _) = ContentCurator::score_segment(boring, 60.0);
    assert!(score_low < 50.0);
}

#[test]
fn test_upstream_sliding_window_highlight_curation() {
    let segments = vec![
        TranscriptSegment {
            id: 0,
            start: 0.0,
            end: 10.0,
            text: "Welcome back to the studio.".to_string(),
            words: vec![],
            speaker: None,
        },
        TranscriptSegment {
            id: 1,
            start: 10.0,
            end: 35.0,
            text: "Why does everyone fall for this shocking money hack?".to_string(),
            words: vec![],
            speaker: None,
        },
        TranscriptSegment {
            id: 2,
            start: 35.0,
            end: 55.0,
            text: "Here is the exact truth on how the algorithm works.".to_string(),
            words: vec![],
            speaker: None,
        },
    ];

    let tr = Transcript::new(segments);
    let top_candidates = ContentCurator::find_viral_candidates(&tr, 15.0, 50.0, 2);

    assert!(!top_candidates.is_empty());
    let best = &top_candidates[0];
    assert!(best.virality_score >= 60.0);
    assert!(best.hook_text.contains("Why does everyone fall"));
}

#[test]
fn test_upstream_llm_structured_prompt_and_response() {
    let tr = Transcript::new(vec![TranscriptSegment {
        id: 0,
        start: 5.0,
        end: 25.0,
        text: "The secret hack to doubling your productivity.".to_string(),
        words: vec![],
        speaker: None,
    }]);

    let prompt = ContentCurator::build_llm_curation_prompt(&tr);
    assert!(prompt.contains("expert viral content producer"));
    assert!(prompt.contains("virality_score"));

    // Test parsing structured LLM response
    let fake_llm_output = r#"[
        {
            "start_time": 5.0,
            "end_time": 25.0,
            "duration": 20.0,
            "hook_text": "The secret hack to doubling your productivity.",
            "virality_score": 92.5,
            "reason": "Direct power word in the opener with actionable payoff"
        }
    ]"#;

    let parsed = ContentCurator::parse_llm_candidates(fake_llm_output).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].virality_score, 92.5);
    assert_eq!(parsed[0].duration, 20.0);
}

#[test]
fn test_upstream_face_tracking_and_boundary_clamping() {
    // 1. Center of 1920x1080 -> 608x1080 centered at 960
    let crop_center = FramingEngine::calculate_9_16_crop(1920, 1080, 960.0);
    assert_eq!(crop_center.width, 608);
    assert_eq!(crop_center.height, 1080);
    assert_eq!(crop_center.x, 960 - 304);
    assert_eq!(crop_center.y, 0);

    // 2. Boundary left clamping
    let crop_left = FramingEngine::calculate_9_16_crop(1920, 1080, 10.0);
    assert_eq!(crop_left.x, 0);

    // 3. Boundary right clamping
    let crop_right = FramingEngine::calculate_9_16_crop(1920, 1080, 1910.0);
    assert_eq!(crop_right.x, 1920 - 608);

    // 4. Face tracker smoothing (EMA)
    let mut smoother = FaceTrackerSmoother::new(0.25);
    let step1 = smoother.update(500.0);
    assert_eq!(step1, 500.0);
    let step2 = smoother.update(600.0);
    assert_eq!(step2, 500.0 + 0.25 * 100.0); // 525.0
}

#[test]
fn test_upstream_podcast_split_screen_filter() {
    let filter = FramingEngine::build_split_screen_filter(1920, 1080, 480.0, 1440.0);
    assert!(filter.contains("scale=1080:960[top]"));
    assert!(filter.contains("scale=1080:960[bottom]"));
    assert!(filter.contains("vstack=inputs=2[outv]"));
}

#[test]
fn test_upstream_kinetic_karaoke_ass_generation() {
    let seg = TranscriptSegment {
        id: 0,
        start: 0.0,
        end: 3.0,
        text: "Make things happen".to_string(),
        words: vec![
            WordTiming::new("Make", 0.0, 0.8),
            WordTiming::new("things", 0.8, 1.8),
            WordTiming::new("happen", 1.8, 3.0),
        ],
        speaker: None,
    };
    let tr = Transcript::new(vec![seg]);
    let style = SubtitleStyle::default();

    let ass = SubtitleEngine::generate_karaoke_ass(&tr, 0.0, 3.0, &style, 3);
    assert!(ass.contains("[Script Info]"));
    assert!(ass.contains("PlayResX: 1080"));
    assert!(ass.contains("PlayResY: 1920"));
    assert!(ass.contains("[V4+ Styles]"));
    assert!(ass.contains("Style: Default,Montserrat ExtraBold"));
    assert!(ass.contains("[Events]"));
    assert!(ass.contains(r"{\k80\t(0,70,\fscx110\fscy110)\t(70,140,\fscx100\fscy100)}MAKE"));
    assert!(ass.contains("HAPPEN"));
}

#[test]
fn test_upstream_cinematic_teaser_filter() {
    let cfg = TeaserConfig {
        duration_seconds: 3.0,
        hook_title: "NEVER DO THIS".to_string(),
        overlay_opacity: 0.5,
        enable_countdown: true,
    };

    let filter = TeaserEngine::build_teaser_filter(&cfg);
    assert!(filter.contains("zoompan=z="));
    assert!(filter.contains("vignette=PI/4"));
    assert!(filter.contains("NEVER DO THIS"));
}

#[test]
fn test_upstream_sidechain_audio_ducking_filter() {
    let cfg = AudioDuckingConfig {
        bgm_volume: 0.18,
        ducking_threshold: 0.10,
        ducking_ratio: 6.0,
        attack_ms: 40.0,
        release_ms: 250.0,
        target_lufs: -16.0,
    };

    let filter = AudioDuckingEngine::build_ducking_filtergraph(&cfg);
    assert!(filter.contains("sidechaincompress=threshold=0.10:ratio=6.0:attack=40.0:release=250.0"));
    assert!(filter.contains("amix=inputs=2"));
    assert!(filter.contains("loudnorm=I=-16.0"));
}

#[test]
fn test_upstream_watermarking_positions() {
    let wm_tr = Watermark::new_text("Brand", WatermarkPosition::TopRight);
    let f_tr = wm_tr.to_ffmpeg_filter();
    assert!(f_tr.contains("x=w-text_w-40"));
    assert!(f_tr.contains("y=60"));

    let wm_bl = Watermark::new_text("Brand", WatermarkPosition::BottomLeft);
    let f_bl = wm_bl.to_ffmpeg_filter();
    assert!(f_bl.contains("x=40"));
    assert!(f_bl.contains("y=h-text_h-60"));
}

#[test]
fn test_upstream_multi_hook_story_mode() {
    let c1 = ViralCandidate {
        start_time: 10.0,
        end_time: 35.0,
        duration: 25.0,
        hook_text: "Top climax hook".to_string(),
        virality_score: 96.0,
        reason: "Highest engagement".to_string(),
    };
    let c2 = ViralCandidate {
        start_time: 60.0,
        end_time: 80.0,
        duration: 20.0,
        hook_text: "Second teaser".to_string(),
        virality_score: 88.0,
        reason: "Micro hook".to_string(),
    };

    let story = StoryEngine::assemble_story(&[c1, c2]).unwrap();
    assert_eq!(story.hook_clips.len(), 1);
    assert_eq!(story.main_climax_clip.virality_score, 96.0);
    assert_eq!(story.total_estimated_duration, 2.5 + 25.0);
}

#[test]
fn test_upstream_social_media_metadata() {
    let meta = MetadataGenerator::generate("Why do 99% of people fail?", "Success");
    assert!(meta.youtube_title.len() <= 60);
    assert!(meta.youtube_title.contains("#shorts"));
    assert!(meta.youtube_tags.contains(&"success".to_string()));
    assert!(meta.tiktok_caption.contains("#fyp"));
    assert!(meta.instagram_caption.contains("Save this for later"));
}

#[test]
fn test_upstream_ffmpeg_render_pipeline() {
    let opts = RenderClipOptions {
        input_path: PathBuf::from("video.mp4"),
        output_path: PathBuf::from("short.mp4"),
        start_time: 10.0,
        end_time: 30.0,
        crop: Some(CropWindow {
            x: 656,
            y: 0,
            width: 608,
            height: 1080,
        }),
        ass_subtitles_path: Some(PathBuf::from("subtitles.ass")),
        watermark: Some(Watermark::new_text(
            "@calljacob",
            WatermarkPosition::TopRight,
        )),
        ..RenderClipOptions::default()
    };

    let cmd = RenderPipeline::build_ffmpeg_command(&opts).unwrap();
    assert_eq!(cmd[0], "ffmpeg");
    assert!(cmd.iter().any(|c| c == "-ss"));
    assert!(cmd.iter().any(|c| c == "10.000"));
    assert!(cmd.iter().any(|c| c == "-to"));
    assert!(cmd.iter().any(|c| c == "30.000"));
    assert!(cmd.iter().any(|c| c.contains("crop=608:1080:656:0")));
    assert!(cmd.iter().any(|c| c.contains("subtitles='subtitles.ass'")));
    assert!(cmd.iter().any(|c| c.contains("drawtext=text='@calljacob'")));
}

#[test]
fn test_upstream_scenedetect_and_matchcut_similarity() {
    let shot_a = [240u8, 30, 30].repeat(64);
    let shot_b = [20u8, 40, 230].repeat(64);
    let black = [0u8, 0, 0].repeat(64);

    let h_a1 = FrameHistogram::from_rgb24(0, 0.0, &shot_a);
    let h_a2 = FrameHistogram::from_rgb24(1, 0.033, &shot_a);
    let h_b1 = FrameHistogram::from_rgb24(2, 0.066, &shot_b);
    let h_blk = FrameHistogram::from_rgb24(3, 0.100, &black);

    assert!(h_a1.cosine_similarity(&h_a2) > 0.99);
    assert!(h_a1.cosine_similarity(&h_b1) < 0.25);

    let cuts = SceneDetector::detect_content_cuts(&[h_a1, h_a2, h_b1.clone(), h_blk.clone()], 30.0, 1);
    assert!(!cuts.is_empty());

    let fades = SceneDetector::detect_luma_fades(&[h_b1, h_blk], 12.0);
    assert_eq!(fades.len(), 1);
    assert_eq!(fades[0].detector_kind, "fade_out");
}
