use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde_json::Value;
use tempfile::TempDir;
use vid_dup_finder_lib::Cropdetect;

use crate::common::tool_data::CommonData;
use crate::common::traits::Search;
use crate::tools::similar_videos::{SimilarVideos, SimilarVideosParameters, VideosEntry};

// Tests are quite limited here, due to the needing of external ffmpeg libraries and video files.
// Just tested is that searching in an empty directory works as expected - no found similar videos

#[test]
fn test_similar_videos_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    let params = SimilarVideosParameters::new(10, false, 15, 10, Cropdetect::Letterbox, false, 0, false, 2);

    let mut finder = SimilarVideos::new(params);
    finder.set_included_paths(vec![path.to_path_buf()]);
    finder.set_recursive_search(true);
    finder.set_use_cache(false);

    let stop_flag = Arc::new(AtomicBool::new(false));
    finder.search(&stop_flag, None);

    let info = finder.get_information();
    assert_eq!(info.number_of_duplicates, 0, "Should find no duplicates in empty directory");
    assert_eq!(info.number_of_groups, 0, "Should find no groups in empty directory");
}

#[test]
fn test_videos_entry_serializes_thumbnail_path() {
    let entry = VideosEntry {
        path: PathBuf::from("video.mp4"),
        size: 123,
        modified_date: 0,
        vhash: Default::default(),
        error: String::new(),
        fps: None,
        codec: None,
        bitrate: None,
        width: None,
        height: None,
        duration: None,
        thumbnail_path: Some(PathBuf::from("thumb.jpg")),
    };

    let serialized = serde_json::to_value(&entry).expect("Serialization should succeed");
    let thumbnail_value = serialized.get("thumbnail_path");

    assert!(thumbnail_value.is_some(), "thumbnail_path should be present in JSON output");
    assert_eq!(thumbnail_value, Some(&Value::String("thumb.jpg".to_string())));
}
