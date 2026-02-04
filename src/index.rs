use crate::helpers::{
    self, ensure_output_dir, normalize_path, validate_file_exists, validate_times,
};
use crate::vid::trim_video_with_rust;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

// check video duration
fn check_video_duration(video_path: &str) -> PyResult<String> {
    // Validate file exists
    validate_file_exists(&video_path)?;
    //normalize paths
    let normalized_path = video_path.replace("\\", "/");
    // let (video_path, output_path) = normalize_paths(&video_path);
    // ensure_output_dir(&output_path)?;

    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-i")
        .arg(&normalized_path)
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("csv=p=0")
        .output()
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("FFprobe failed: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "FFprobe failed: {}",
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
// chunk videos
#[allow(dead_code)]
fn chunk_videos_with_rust(
    video_path: String,
    start_time: f64,
    end_time: f64,
    output_path: String,
) -> PyResult<String> {
    //validate file exists
    validate_file_exists(&video_path)?;
    // normalize paths
    let (video_path, output_path) = normalize_path(&video_path, &output_path);
    ensure_output_dir(&output_path)?;

    // Handle existing output file
    if Path::new(&output_path).exists() {
        if let Err(_) = fs::remove_file(&output_path) {
            // If removal fails, try with _1 suffix
            if let Some((base, ext)) = output_path.rsplit_once('.') {
                let new_path = format!("{}_1.{}", base, ext);
                return trim_video_with_rust(video_path, start_time, end_time, new_path);
            }
        }
    }

    // Validate times
    validate_times(start_time, end_time)?;

    // let total_duration = check_video_duration(video_path, output_path);

    // Compare duration numerically for safety and correctness
    let chunk_duration_secs: f64 = 30.0;

    let duration_str = check_video_duration(&video_path)?;

    let video_duration_secs = duration_str.parse::<f64>().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to parse video duration '{}' as a number: {}",
            duration_str, e
        ))
    })?;

    if video_duration_secs > chunk_duration_secs {
        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(&video_path)
            .arg("-c:v")
            .arg("libx264")
            .arg("-crf")
            .arg("18")
            .arg("-preset")
            .arg("fast")
            .arg("-c:a")
            .arg("aac")
            .arg("-map")
            .arg("0:v")
            .arg("-map")
            .arg("0:a")
            .arg("-f")
            .arg("segment")
            .arg("-segment_time")
            .arg(&chunk_duration_secs.to_string())
            .arg("-reset_timestamps")
            .arg("1")
            .arg(&output_path)
            .output()
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("FFmpeg failed: {}", e))
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "FFmpeg failed: {}",
                stderr
            )));
        }
        Ok(output_path)
    } else {
        Ok(video_path)
    }
}

// extract thumbnails
#[allow(dead_code)]
#[pyfunction]
pub fn extract_thumbnails(
    video_path: String,
    _start_time: f64,
    output_path: String,
) -> PyResult<String> {
    // Validate input video exists
    validate_file_exists(&video_path)?;

    // Normalize paths and ensure output directory exists
    let (normalized_video_path, normalized_output_path) = normalize_path(&video_path, &output_path);
    ensure_output_dir(&normalized_output_path)?;

    // Compute safe timestamps: start, middle, end using ffprobe duration
    let duration_str = check_video_duration(&normalized_video_path)?;
    let duration = duration_str.parse::<f64>().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Failed to parse video duration '{}' as a number: {}",
            duration_str, e
        ))
    })?;

    // Safeguards: handle very short or zero-duration videos
    let start_ts: f64;
    let middle_ts: f64;
    let end_ts: f64;

    if duration.is_finite() && duration > 0.0 {
        // Use small offsets to avoid edge decoding issues
        let epsilon = 0.1_f64; // 100ms into the video
        start_ts = if duration > epsilon { epsilon } else { 0.0 };
        middle_ts = (duration / 2.0).max(0.0);
        let end_offset = 0.2_f64; // 200ms before the end
        end_ts = if duration > end_offset {
            (duration - end_offset).max(0.0)
        } else {
            // If the video is extremely short, fall back near start
            (duration * 0.8).max(0.0)
        };
    } else {
        // When duration cannot be determined, default to three safe probes
        start_ts = 0.0;
        middle_ts = 1.0;
        end_ts = 2.0;
    }

    // Build output file paths (JPEGs)
    let start_thumb = format!("{}/start.jpg", &normalized_output_path);
    let mid_thumb = format!("{}/middle.jpg", &normalized_output_path);
    let end_thumb = format!("{}/end.jpg", &normalized_output_path);

    // Helper to run ffmpeg for a single frame extraction
    let run_ffmpeg_thumbnail = |ts: f64, out_path: &str| -> PyResult<()> {
        let output = Command::new("ffmpeg")
            .arg("-y") // overwrite if exists
            .arg("-ss")
            .arg(ts.to_string())
            .arg("-i")
            .arg(&normalized_video_path)
            .arg("-frames:v")
            .arg("1")
            .arg("-q:v")
            .arg("2") // high quality thumbnail
            .arg(out_path)
            .output()
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("FFmpeg failed: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "FFmpeg failed: {}",
                stderr
            )));
        }
        Ok(())
    };

    // Extract thumbnails at start, middle, and end
    run_ffmpeg_thumbnail(start_ts, &start_thumb)?;
    run_ffmpeg_thumbnail(middle_ts, &mid_thumb)?;
    run_ffmpeg_thumbnail(end_ts, &end_thumb)?;

    // Return the three generated paths as a comma-separated string
    Ok(format!("{},{},{}", start_thumb, mid_thumb, end_thumb))
}

// chunk multiple videos
#[pyfunction]
#[allow(dead_code)]
pub fn chunk_multiple_videos_with_rust(
    video_paths: Vec<String>,
    // Treat this as an output directory, not a filename.
    output_dir: String,
    chunk_duration_secs: f64,
) -> PyResult<Vec<String>> {
    let mut results = Vec::with_capacity(video_paths.len());

    // Normalize output directory and ensure it exists
    let normalized_out_dir = output_dir.replace("\\", "/");
    ensure_output_dir(&normalized_out_dir)?;

    for vp in video_paths {
        // Validate and normalize input
        validate_file_exists(&vp)?;
        let normalized_vp = vp.replace("\\", "/");

        // Duration check
        let duration_str = check_video_duration(&normalized_vp)?;
        let video_duration_secs = duration_str.parse::<f64>().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse video duration '{}' as a number: {}",
                duration_str, e
            ))
        })?;

        if video_duration_secs > chunk_duration_secs {
            // Build per-video output template: {out_dir}/{base}_chunk_%03d.mp4
            let base_name = Path::new(&normalized_vp)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("chunk");
            let output_template = format!("{}/{}_chunk_%03d.mp4", &normalized_out_dir, base_name);

            let output = Command::new("ffmpeg")
                .arg("-i")
                .arg(&normalized_vp)
                .arg("-c:v")
                .arg("libx264")
                .arg("-crf")
                .arg("18")
                .arg("-preset")
                .arg("fast")
                .arg("-c:a")
                .arg("aac")
                .arg("-map")
                .arg("0:v")
                .arg("-map")
                .arg("0:a")
                .arg("-f")
                .arg("segment")
                .arg("-segment_time")
                .arg(&chunk_duration_secs.to_string())
                .arg("-reset_timestamps")
                .arg("1")
                .arg(&output_template)
                .output()
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "FFmpeg failed: {}",
                        e
                    ))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "FFmpeg failed: {}",
                    stderr
                )));
            }

            // Record template used for this input
            results.push(output_template);
        } else {
            // Too short to chunk – return original path
            results.push(normalized_vp);
        }
    }

    Ok(results)
}

// write a  function where first
// chunking is done
// then audio extraction and frame extraction is multithreaded
// and returned for python use
#[pyfunction]
pub fn rust_indexer(
    py: Python<'_>,
    video_paths: Vec<String>,
    chunk_duration_secs: f64,
    output_dir: String,
) -> PyResult<Vec<String>> {
    // Normalize paths (directories are created by Python before calling this function)
    let normalized_out_dir = output_dir.replace("\\", "/");
    let chunks_dir = format!("{}/chunks", &normalized_out_dir);
    let audio_dir = format!("{}/audio", &normalized_out_dir);
    let thumbnails_dir = format!("{}/thumbnails", &normalized_out_dir);

    // Step 1: Chunk all videos (sequential - need to know chunk paths before parallel processing)
    let mut all_chunk_paths: Vec<String> = Vec::new();

    for vp in &video_paths {
        validate_file_exists(vp)?;
        let normalized_vp = vp.replace("\\", "/");

        // Get video duration
        let duration_str = check_video_duration(&normalized_vp)?;
        let video_duration_secs = duration_str.parse::<f64>().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to parse video duration '{}' as a number: {}",
                duration_str, e
            ))
        })?;

        let base_name = Path::new(&normalized_vp)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video");

        if video_duration_secs > chunk_duration_secs {
            // Chunk the video
            let output_template = format!("{}/{}_chunk_%03d.mp4", &chunks_dir, base_name);

            let output = Command::new("ffmpeg")
                .arg("-y")
                .arg("-i")
                .arg(&normalized_vp)
                .arg("-c:v")
                .arg("libx264")
                .arg("-crf")
                .arg("18")
                .arg("-preset")
                .arg("fast")
                .arg("-c:a")
                .arg("aac")
                .arg("-map")
                .arg("0:v")
                .arg("-map")
                .arg("0:a?") // optional audio stream
                .arg("-f")
                .arg("segment")
                .arg("-segment_time")
                .arg(&chunk_duration_secs.to_string())
                .arg("-reset_timestamps")
                .arg("1")
                .arg(&output_template)
                .output()
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "FFmpeg chunking failed: {}",
                        e
                    ))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "FFmpeg chunking failed: {}",
                    stderr
                )));
            }

            // Find all generated chunk files
            let chunk_pattern = format!("{}/{}_chunk_", &chunks_dir, base_name);
            if let Ok(entries) = fs::read_dir(&chunks_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(path_str) = path.to_str() {
                        let normalized = path_str.replace("\\", "/");
                        if normalized.starts_with(&chunk_pattern) && normalized.ends_with(".mp4") {
                            all_chunk_paths.push(normalized);
                        }
                    }
                }
            }
        } else {
            all_chunk_paths.push(normalized_vp);
        }
    }

    // Sort chunk paths for consistent ordering
    all_chunk_paths.sort();

    // Step 2: Process chunks in parallel (extract audio + thumbnails)
    let audio_dir_clone = audio_dir.clone();
    let thumbnails_dir_clone = thumbnails_dir.clone();

    let results: Result<Vec<String>, String> = py.detach(|| {
        all_chunk_paths
            .par_iter()
            .map(|chunk_path| {
                // Create unique subdirectory for this chunk's thumbnails
                let chunk_name = Path::new(chunk_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("chunk");

                let chunk_thumb_dir = format!("{}/{}", &thumbnails_dir_clone, chunk_name);

                // Extract audio and thumbnails in parallel for this chunk
                let audio_result = extract_audio_internal(chunk_path, &audio_dir_clone);
                let thumb_result = extract_thumbnails_internal(chunk_path, 0.0, &chunk_thumb_dir);

                // Combine results
                match (audio_result, thumb_result) {
                    (Ok(audio_path), Ok(thumb_paths)) => {
                        // Return as JSON-like string: chunk|audio|thumbnails
                        Ok(format!("{}|{}|{}", chunk_path, audio_path, thumb_paths))
                    }
                    (Err(e), _) => {
                        Err(format!("Audio extraction failed for {}: {}", chunk_path, e))
                    }
                    (_, Err(e)) => Err(format!(
                        "Thumbnail extraction failed for {}: {}",
                        chunk_path, e
                    )),
                }
            })
            .collect()
    });

    // Convert Result<Vec<String>, String> to PyResult<Vec<String>>
    results.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

// Helper function to check if video has audio stream
fn has_audio_stream(video_path: &str) -> bool {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("a")
        .arg("-show_entries")
        .arg("stream=codec_type")
        .arg("-of")
        .arg("csv=p=0")
        .arg(video_path)
        .output();

    match output {
        Ok(o) => !String::from_utf8_lossy(&o.stdout).trim().is_empty(),
        Err(_) => false,
    }
}

// extract multiple audios internal helper
fn extract_audio_internal(video_path: &str, output_dir: &str) -> Result<String, String> {
    let normalized_vp = video_path.replace("\\", "/");
    let normalized_out_dir = output_dir.replace("\\", "/");

    ensure_output_dir(&normalized_out_dir).map_err(|e| e.to_string())?;
    validate_file_exists(&normalized_vp).map_err(|e| e.to_string())?;

    if !has_audio_stream(&normalized_vp) {
        return Ok("NO_AUDIO".to_string());
    }

    let video_filename = Path::new(&normalized_vp)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Could not extract filename from path: {}", normalized_vp))?;

    let mut output_path = format!("{}/{}.mp3", normalized_out_dir, video_filename);

    if Path::new(&output_path).exists() {
        if fs::remove_file(&output_path).is_err() {
            output_path = format!("{}/{}_1.mp3", normalized_out_dir, video_filename);
        }
    }

    let (codec, bitrate_or_samplerate) = helpers::get_audio_encoding_params(&output_path);

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i")
        .arg(&normalized_vp)
        .arg("-vn")
        .arg("-acodec")
        .arg(&codec);

    if codec == "pcm_s16le" {
        cmd.arg("-ar").arg(&bitrate_or_samplerate);
    } else if codec != "flac" {
        cmd.arg("-b:a").arg(&bitrate_or_samplerate);
    }

    cmd.arg(&output_path);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg failed for {}: {}", normalized_vp, stderr));
    }
    Ok(output_path)
}

// extract thumbnails
fn extract_thumbnails_internal(
    video_path: &str,
    _start_time: f64,
    output_path: &str,
) -> Result<String, String> {
    // Validate input video exists
    validate_file_exists(video_path).map_err(|e| e.to_string())?;

    // Normalize paths
    let (normalized_video_path, normalized_output_path) = normalize_path(&video_path, &output_path);

    // Create the thumbnail subdirectory for this specific chunk
    fs::create_dir_all(&normalized_output_path).map_err(|e| {
        format!(
            "Failed to create thumbnail directory {}: {}",
            normalized_output_path, e
        )
    })?;

    // Compute safe timestamps: start, middle, end using ffprobe duration
    let duration_str = check_video_duration(&normalized_video_path).map_err(|e| e.to_string())?;

    // Now you can parse it
    let duration = duration_str.parse::<f64>().map_err(|e| {
        format!(
            "Failed to parse video duration '{}' as a number: {}",
            duration_str, e
        )
    })?;

    // Safeguards: handle very short or zero-duration videos
    let start_ts: f64;
    let middle_ts: f64;
    let end_ts: f64;

    if duration.is_finite() && duration > 0.0 {
        // Use small offsets to avoid edge decoding issues
        let epsilon = 0.1_f64; // 100ms into the video
        start_ts = if duration > epsilon { epsilon } else { 0.0 };
        middle_ts = (duration / 2.0).max(0.0);
        let end_offset = 0.2_f64; // 200ms before the end
        end_ts = if duration > end_offset {
            (duration - end_offset).max(0.0)
        } else {
            // If the video is extremely short, fall back near start
            (duration * 0.8).max(0.0)
        };
    } else {
        // When duration cannot be determined, default to three safe probes
        start_ts = 0.0;
        middle_ts = 1.0;
        end_ts = 2.0;
    }

    // Build output file paths (JPEGs)
    let start_thumb = format!("{}/start.jpg", &normalized_output_path);
    let mid_thumb = format!("{}/middle.jpg", &normalized_output_path);
    let end_thumb = format!("{}/end.jpg", &normalized_output_path);

    // Helper to run ffmpeg for a single frame extraction
    let run_ffmpeg_thumbnail = |ts: f64, out_path: &str| -> Result<(), String> {
        let output = Command::new("ffmpeg")
            .arg("-y") // overwrite if exists
            .arg("-ss")
            .arg(ts.to_string())
            .arg("-i")
            .arg(&normalized_video_path)
            .arg("-frames:v")
            .arg("1")
            .arg("-update")
            .arg("1")
            .arg("-q:v")
            .arg("2") // high quality thumbnail
            .arg(out_path)
            .output()
            .map_err(|e| format!("FFmpeg failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("FFmpeg failed: {}", stderr));
        }
        Ok(())
    };

    // Extract thumbnails at start, middle, and end
    run_ffmpeg_thumbnail(start_ts, &start_thumb)?;
    run_ffmpeg_thumbnail(middle_ts, &mid_thumb)?;
    run_ffmpeg_thumbnail(end_ts, &end_thumb)?;

    // Return the three generated paths as a comma-separated string
    Ok(format!("{},{},{}", start_thumb, mid_thumb, end_thumb))
}
