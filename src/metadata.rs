use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMetadata {
    pub youtube_title: String,
    pub youtube_description: String,
    pub youtube_tags: Vec<String>,
    pub tiktok_caption: String,
    pub instagram_caption: String,
}

pub struct MetadataGenerator;

impl MetadataGenerator {
    /// Generates high-converting cross-platform titles, descriptions, and tags from the hook text.
    pub fn generate(hook_text: &str, raw_topic: &str) -> SocialMetadata {
        let clean_hook = hook_text.trim();
        let topic = if raw_topic.is_empty() { "Viral" } else { raw_topic.trim() };

        // 1. YouTube Shorts title: Punchy, under 60 chars with #shorts
        let yt_title = if clean_hook.len() > 50 {
            format!("{}... #shorts", &clean_hook[..47])
        } else {
            format!("{} #shorts", clean_hook)
        };

        // 2. YouTube description
        let yt_desc = format!(
            "{}\n\n👉 Subscribe for more insightful clips!\n\n#shorts #{} #viral #trending #podcast",
            clean_hook,
            topic.to_lowercase().replace(' ', "")
        );

        // 3. YouTube tags
        let yt_tags = vec![
            "shorts".to_string(),
            topic.to_lowercase(),
            "podcast clips".to_string(),
            "viral moments".to_string(),
            "life hacks".to_string(),
            "trending".to_string(),
        ];

        // 4. TikTok caption with hashtags (under 150 chars)
        let tiktok_caption = format!(
            "{} 🤯 #fyp #foryou #{} #viral #trending #mindblown",
            clean_hook,
            topic.to_lowercase().replace(' ', "")
        );

        // 5. Instagram Reels caption with engagement CTA
        let instagram_caption = format!(
            "{}\n.\n.\nDid you know this? Let us know in the comments below! 👇\n\nSave this for later 📌\n\n#reels #{} #viralreels #growth #dailyknowledge",
            clean_hook,
            topic.to_lowercase().replace(' ', "")
        );

        SocialMetadata {
            youtube_title: yt_title,
            youtube_description: yt_desc,
            youtube_tags: yt_tags,
            tiktok_caption,
            instagram_caption,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_generation() {
        let meta = MetadataGenerator::generate("Did you know the secret mistake with money?", "Finance");
        assert!(meta.youtube_title.contains("#shorts"));
        assert!(meta.youtube_tags.contains(&"finance".to_string()));
        assert!(meta.tiktok_caption.contains("#fyp"));
        assert!(meta.instagram_caption.contains("Save this for later"));
    }
}
