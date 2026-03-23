use {
    crate::{Segment, YoMama, YoMamaJoke},
    color_eyre::eyre::eyre,
    cpal::{
        SampleFormat,
        traits::{HostTrait, StreamTrait},
    },
    mp3lame_encoder::{Birtate, Builder, DualPcm, FlushNoGap, MonoPcm, Quality},
    polyprompt::{Backend, PolyOption, PolyPrompt},
    rodio::DeviceTrait,
    std::{
        fs,
        io::Write as _,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    },
};

const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");

const CATEGORIES: &[&str] = &["base", "adjective", "clause", "action", "reaction"];

fn const_stem(clip_name: &str) -> String {
    Path::new(clip_name)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_uppercase()
        .replace(['-', '.', ' '], "_")
}

fn next_filename(category_dir: &Path) -> PathBuf {
    let existing = fs::read_dir(category_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.strip_suffix(".mp3")?.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(0);

    category_dir.join(format!("{:02}.mp3", existing + 1))
}

fn read_line_trimmed(label: &str) -> color_eyre::Result<String> {
    print!("{label}");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn select_segment_type(prompt: &PolyPrompt) -> color_eyre::Result<Segment> {
    let options: Vec<_> = Segment::ALL
        .iter()
        .enumerate()
        .map(|(i, s)| PolyOption::new(s.name(), i))
        .collect();
    let idx = prompt
        .select::<usize>("Select segment category")
        .with_page_size(20)
        .with_options(options)
        .run()?;
    Ok(Segment::ALL[idx])
}

fn select_clip_from(
    prompt: &PolyPrompt,
    segment: Segment,
) -> color_eyre::Result<&'static crate::SegmentClip> {
    let clips = segment.all_clips();
    let prefix = segment.field_name().to_uppercase();
    let options: Vec<_> = clips
        .iter()
        .enumerate()
        .map(|(i, clip)| {
            let stem = const_stem(clip.name);
            PolyOption::new(
                format!("{} ({prefix}_{stem}) [0x{:016x}]", clip.name, clip.hash),
                i,
            )
        })
        .collect();
    let idx = prompt
        .select::<usize>(&format!("Select {} segment", segment.name()))
        .with_page_size(20)
        .with_options(options)
        .run()?;
    Ok(clips[idx])
}

pub fn list_segments() {
    for &seg_type in Segment::ALL {
        let clips = seg_type.all_clips();
        let prefix = seg_type.field_name().to_uppercase();
        println!("\n=== {} ({} files) ===", seg_type.name(), clips.len());
        for clip in clips {
            let stem = const_stem(clip.name);
            let const_name = format!("{prefix}_{stem}");
            println!("  {const_name:<28} {:<25} 0x{:016x}", clip.name, clip.hash);
        }
    }
    println!();
}

pub fn play_specific_joke() -> color_eyre::Result<()> {
    let prompt = PolyPrompt::new(Backend::Bearask);

    let base = select_clip_from(&prompt, Segment::Base)?;
    let adjective = select_clip_from(&prompt, Segment::Adjective)?;
    let clause = select_clip_from(&prompt, Segment::Clause)?;
    let action = select_clip_from(&prompt, Segment::Action)?;
    let reaction = select_clip_from(&prompt, Segment::Reaction)?;

    let joke = YoMamaJoke {
        base,
        adjective,
        clause,
        action,
        reaction,
    };

    println!(
        "Playing: {} + {} + {} + {} + {}",
        base.name, adjective.name, clause.name, action.name, reaction.name
    );

    YoMama::play_joke(joke);
    Ok(())
}

pub fn play_single_segment() -> color_eyre::Result<()> {
    let prompt = PolyPrompt::new(Backend::Bearask);
    let segment = select_segment_type(&prompt)?;
    let clip = select_clip_from(&prompt, segment)?;

    println!("Playing: {} / {}", segment.name(), clip.name);
    crate::play_audio(clip.data, 5.0);
    Ok(())
}

pub fn inspect_random_joke() {
    let joke = YoMamaJoke::generate();
    println!("\nRandom Joke Composition:");
    println!("  Base:      {}", joke.base.name);
    println!("  Adjective: {}", joke.adjective.name);
    println!("  Clause:    {}", joke.clause.name);
    println!("  Action:    {}", joke.action.name);
    println!("  Reaction:  {}", joke.reaction.name);

    let effects = crate::eastereggs::register(&joke);
    if effects.is_empty() {
        println!("  Easter eggs triggered: none");
    } else {
        println!("  Easter eggs triggered: {}", effects.len());
        for (seg, phase, _) in &effects {
            println!("    - {seg:?} [{phase:?}]");
        }
    }
    println!();
}

pub fn stress_test() -> color_eyre::Result<()> {
    let input = read_line_trimmed("Number of jokes to play: ")?;
    let count: usize = input.parse().unwrap_or(5);

    for i in 0..count {
        println!("\n--- Joke {}/{count} ---", i + 1);
        let joke = YoMamaJoke::generate();
        println!(
            "  {} + {} + {} + {} + {}",
            joke.base.name,
            joke.adjective.name,
            joke.clause.name,
            joke.action.name,
            joke.reaction.name
        );
        YoMama::play_joke(joke);
    }

    println!("\nStress test complete.");
    Ok(())
}

fn overlays_dir() -> PathBuf {
    Path::new(PROJECT_DIR).join("assets").join("overlays")
}

fn parse_include_file_entries(content: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut in_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("include_file!") {
            in_block = true;
            continue;
        }
        if in_block {
            if trimmed == "}" {
                break;
            }
            if let Some((name, path)) = trimmed.split_once("=>") {
                let name = name.trim().to_string();
                let path = path.trim().trim_matches('"').to_string();
                entries.push((name, path));
            }
        }
    }

    entries
}

fn read_segments_rs() -> color_eyre::Result<(String, Vec<(String, String)>)> {
    let path = Path::new(PROJECT_DIR).join("src").join("segments.rs");
    let content = fs::read_to_string(&path)?;
    let entries = parse_include_file_entries(&content);
    Ok((content, entries))
}

pub fn list_overlays() {
    let (_, registered) = read_segments_rs().unwrap_or_default();
    let dir = overlays_dir();

    println!("\n=== Overlay Images ===");

    if let Ok(mut entries) =
        fs::read_dir(&dir).map(|rd| rd.filter_map(|e| e.ok()).collect::<Vec<_>>())
    {
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let fname = entry.file_name().to_string_lossy().into_owned();
            let rel_path = format!("../assets/overlays/{fname}");

            let const_name = registered
                .iter()
                .find(|(_, path)| *path == rel_path)
                .map(|(name, _)| name.as_str());

            if let Some(name) = const_name {
                println!("  {name:<25} {fname}");
            } else {
                let suggested = Path::new(&fname)
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_uppercase()
                    + "_SPR";
                println!("  (unregistered)          {fname}  -> suggested const: {suggested}");
            }
        }
    } else {
        println!("  No overlays directory found.");
    }
    println!();
}

pub fn download_overlay() -> color_eyre::Result<()> {
    let prompt = PolyPrompt::new(Backend::Bearask);

    let url = read_line_trimmed("Enter image URL: ")?;
    if url.is_empty() {
        return Err(eyre!("No URL provided"));
    }

    let default_fname = url
        .rsplit('/')
        .next()
        .and_then(|s| {
            let s = s.split('?').next().unwrap_or(s);
            if s.contains('.') { Some(s) } else { None }
        })
        .unwrap_or("overlay.jpg");

    let filename = read_line_trimmed(&format!("Save as filename [{default_fname}]: "))?;
    let filename = if filename.is_empty() {
        default_fname.to_string()
    } else {
        filename
    };

    let dir = overlays_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    let output_path = dir.join(&filename);

    println!("Downloading...");
    let status = std::process::Command::new("curl")
        .args([
            "-L",
            "--fail",
            "--silent",
            "--show-error",
            "-o",
            &output_path.to_string_lossy(),
            &url,
        ])
        .status()?;

    if !status.success() {
        return Err(eyre!("Download failed (curl exit code: {status})"));
    }

    let size = fs::metadata(&output_path)?.len();
    println!("Downloaded {} ({size} bytes)", output_path.display());

    let stem = Path::new(&filename)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_uppercase()
        .replace(['-', '.', ' '], "_");
    let const_name = format!("{stem}_SPR");

    let should_register = prompt
        .confirm(format!("Register as {const_name} in segments.rs?"))
        .with_default(true)
        .run()?;

    if should_register {
        register_overlay_in_segments_rs(&const_name, &filename)?;
        println!("Registered as {const_name}. Rebuild to include it.");
    }

    Ok(())
}

fn register_overlay_in_segments_rs(const_name: &str, filename: &str) -> color_eyre::Result<()> {
    let segments_path = Path::new(PROJECT_DIR).join("src").join("segments.rs");
    let content = fs::read_to_string(&segments_path)?;

    let new_entry = format!("    {const_name} => \"../assets/overlays/{filename}\"");

    let mut new_content = String::new();
    let mut in_include_file = false;
    let mut inserted = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("include_file!") {
            in_include_file = true;
        }
        if in_include_file && !inserted && trimmed == "}" {
            new_content.push_str(&new_entry);
            new_content.push('\n');
            inserted = true;
        }
        new_content.push_str(line);
        new_content.push('\n');
    }

    if !inserted {
        return Err(eyre!(
            "Could not find include_file! block closing brace in segments.rs"
        ));
    }

    fs::write(&segments_path, new_content)?;
    Ok(())
}

pub fn preview_overlay() -> color_eyre::Result<()> {
    let prompt = PolyPrompt::new(Backend::Bearask);
    let dir = overlays_dir();

    let mut files: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();

    if files.is_empty() {
        println!("No overlay files found.");
        return Ok(());
    }

    let options: Vec<_> = files
        .iter()
        .enumerate()
        .map(|(i, f)| PolyOption::new(f.as_str(), i))
        .collect();

    let idx = prompt
        .select::<usize>("Select overlay to preview")
        .with_options(options)
        .run()?;

    let file_path = dir.join(&files[idx]);
    let image_data = fs::read(&file_path)?;

    let (cx, cy) = (0, 0);

    let is_animated = files[idx].ends_with(".gif") || files[idx].ends_with(".webp");

    if is_animated {
        let _overlay = crate::lmao::AnimatedOverlay::new(&image_data, cx, cy, 255, None)
            .map_err(|e| eyre!("Failed to create animated overlay: {e}"))?;
        prompt
            .confirm("Overlay displayed. Press enter to dismiss.")
            .with_default(true)
            .run()?;
    } else {
        let _overlay = crate::lmao::TransparentOverlay::new(&image_data, cx, cy, 255, None)
            .map_err(|e| eyre!("Failed to create overlay: {e}"))?;
        prompt
            .confirm("Overlay displayed. Press enter to dismiss.")
            .with_default(true)
            .run()?;
    }

    Ok(())
}

struct ExistingEgg {
    hash: u64,
    phase: String,
    effect: String,
}

fn parse_existing_easter_eggs() -> Vec<ExistingEgg> {
    let path = Path::new(PROJECT_DIR).join("src").join("eastereggs.rs");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut eggs = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }

        let Some(hash_start) = trimmed.find("== 0x") else {
            continue;
        };
        let hex_start = hash_start + 5;
        let Some(hex_len) = trimmed[hex_start..].find("_u64") else {
            continue;
        };
        let Ok(hash) = u64::from_str_radix(&trimmed[hex_start..hex_start + hex_len], 16) else {
            continue;
        };

        let after_hash = &trimmed[hex_start + hex_len + 4..];
        let phase = if after_hash.contains("[After]") {
            "After"
        } else if after_hash.contains("[Before]") {
            "Before"
        } else {
            "During"
        };

        let Some(effect_start) = after_hash.find("=> ") else {
            continue;
        };
        let effect = after_hash[effect_start + 3..].trim_end_matches(';').trim();

        eggs.push(ExistingEgg {
            hash,
            phase: phase.to_string(),
            effect: effect.to_string(),
        });
    }
    eggs
}

fn egg_count_by_hash(eggs: &[ExistingEgg]) -> std::collections::HashMap<u64, usize> {
    let mut counts = std::collections::HashMap::new();
    for egg in eggs {
        *counts.entry(egg.hash).or_insert(0) += 1;
    }
    counts
}

pub fn register_easter_egg() -> color_eyre::Result<()> {
    let prompt = PolyPrompt::new(Backend::Bearask);

    let existing = parse_existing_easter_eggs();
    let counts = egg_count_by_hash(&existing);

    let mut seg_with_counts: Vec<_> = Segment::ALL
        .iter()
        .map(|&seg| {
            let total: usize = seg
                .all_clips()
                .iter()
                .map(|c| counts.get(&c.hash).copied().unwrap_or(0))
                .sum();
            (seg, total)
        })
        .collect();
    seg_with_counts.sort_by_key(|&(_, total)| total);

    let seg_options: Vec<_> = seg_with_counts
        .iter()
        .enumerate()
        .map(|(i, (seg, total))| {
            let clips_without = seg
                .all_clips()
                .iter()
                .filter(|c| !counts.contains_key(&c.hash))
                .count();
            let label = if *total == 0 {
                format!("{} (no easter eggs)", seg.name())
            } else {
                format!(
                    "{} ({total} easter egg{}, {clips_without} clip{} without)",
                    seg.name(),
                    if *total == 1 { "" } else { "s" },
                    if clips_without == 1 { "" } else { "s" },
                )
            };
            PolyOption::new(label, i)
        })
        .collect();

    let seg_idx = prompt
        .select::<usize>("Select trigger segment type")
        .with_page_size(20)
        .with_options(seg_options)
        .run()?;
    let segment = seg_with_counts[seg_idx].0;

    let clips = segment.all_clips();
    let prefix = segment.field_name().to_uppercase();

    let mut clip_with_counts: Vec<_> = clips
        .iter()
        .enumerate()
        .map(|(orig_idx, clip)| {
            let count = counts.get(&clip.hash).copied().unwrap_or(0);
            (orig_idx, *clip, count)
        })
        .collect();
    clip_with_counts.sort_by_key(|&(_, _, count)| count);

    let clip_options: Vec<_> = clip_with_counts
        .iter()
        .enumerate()
        .map(|(i, &(_, clip, count))| {
            let stem = const_stem(clip.name);
            let suffix = match count {
                0 => " (no easter eggs)".to_string(),
                n => format!(" ({n} easter egg{})", if n == 1 { "" } else { "s" }),
            };
            PolyOption::new(
                format!(
                    "{} ({prefix}_{stem}) [0x{:016x}]{suffix}",
                    clip.name, clip.hash
                ),
                i,
            )
        })
        .collect();

    let clip_sel = prompt
        .select::<usize>(&format!("Select {} segment", segment.name()))
        .with_page_size(20)
        .with_options(clip_options)
        .run()?;
    let clip = clip_with_counts[clip_sel].1;

    let phase_options = vec![
        PolyOption::new("During (while segment plays)", 0usize),
        PolyOption::new("Before (before segment plays)", 1usize),
        PolyOption::new("After (after segment ends)", 2usize),
    ];
    let phase_idx = prompt
        .select::<usize>("Select phase")
        .with_options(phase_options)
        .run()?;
    let phase_str = match phase_idx {
        0 => "During",
        1 => "Before",
        _ => "After",
    };

    let effect_options = vec![
        PolyOption::new("Image overlay", 0usize),
        PolyOption::new("Animated overlay (GIF/WebP)", 1usize),
        PolyOption::new("Play audio", 2usize),
        PolyOption::new("Crash", 3usize),
    ];
    let effect_idx = prompt
        .select::<usize>("Select effect type")
        .with_options(effect_options)
        .with_page_size(20)
        .run()?;

    let effect_str = match effect_idx {
        0 | 1 => {
            let sprite_const = select_or_download_overlay(&prompt)?;

            let scale_input = read_line_trimmed("Scale factor (leave empty for none): ")?;

            let keyword = if effect_idx == 0 {
                "overlay"
            } else {
                "animated_overlay"
            };

            if scale_input.is_empty() {
                format!("{keyword}({sprite_const})")
            } else {
                format!("{keyword}({sprite_const}, scale: {scale_input})")
            }
        }
        2 => {
            let (_, registered) = read_segments_rs()?;
            let audio_entries: Vec<_> = registered
                .iter()
                .filter(|(_, path)| path.ends_with(".mp3"))
                .collect();

            let audio_const = if audio_entries.is_empty() {
                read_line_trimmed("Audio constant name (e.g., OHCOMEON): ")?
            } else {
                let mut options: Vec<_> = audio_entries
                    .iter()
                    .enumerate()
                    .map(|(i, (name, _))| PolyOption::new(name.as_str(), i))
                    .collect();
                options.push(PolyOption::new("Enter manually", audio_entries.len()));

                let idx = prompt
                    .select::<usize>("Select audio constant")
                    .with_options(options)
                    .run()?;

                if idx == audio_entries.len() {
                    read_line_trimmed("Audio constant name: ")?
                } else {
                    audio_entries[idx].0.clone()
                }
            };

            let volume = read_line_trimmed("Volume (e.g., 5.0): ")?;
            let volume = if volume.is_empty() {
                "5.0".to_string()
            } else {
                volume
            };

            format!("audio({audio_const}, volume: {volume})")
        }
        _ => "crash".to_string(),
    };

    let is_duplicate = existing
        .iter()
        .any(|egg| egg.hash == clip.hash && egg.phase == phase_str && egg.effect == effect_str);

    if is_duplicate {
        println!("\nThis exact easter egg already exists! Skipping.");
        return Ok(());
    }

    let field = segment.field_name();

    let phase_annotation = if phase_str == "During" {
        String::new()
    } else {
        format!(" [{phase_str}]")
    };

    let entry_code = format!(
        "        {}.{field} == 0x{:016x}_u64{phase_annotation} => {effect_str};",
        segment.name(),
        clip.hash,
    );

    println!("\nGenerated easter egg entry:");
    println!("  {entry_code}");

    let should_add = prompt
        .confirm("Add to eastereggs.rs?")
        .with_default(true)
        .run()?;

    if should_add {
        insert_easter_egg_entry(&entry_code)?;
        println!("Added! Rebuild to activate the easter egg.");
    }

    Ok(())
}

fn select_or_download_overlay(prompt: &PolyPrompt) -> color_eyre::Result<String> {
    let (_, registered) = read_segments_rs()?;
    let overlay_entries: Vec<_> = registered
        .iter()
        .filter(|(_, path)| path.contains("overlays/"))
        .collect();

    let mut options: Vec<_> = overlay_entries
        .iter()
        .enumerate()
        .map(|(i, (name, path))| {
            let fname = Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            PolyOption::new(format!("{name} ({fname})"), i)
        })
        .collect();
    options.push(PolyOption::new(
        "Download new overlay from URL".to_string(),
        overlay_entries.len(),
    ));

    let idx = prompt
        .select::<usize>("Select overlay image")
        .with_page_size(20)
        .with_options(options)
        .run()?;

    if idx == overlay_entries.len() {
        let url = read_line_trimmed("Enter image URL: ")?;
        if url.is_empty() {
            return Err(eyre!("No URL provided"));
        }

        let default_fname = url
            .rsplit('/')
            .next()
            .and_then(|s| {
                let s = s.split('?').next().unwrap_or(s);
                if s.contains('.') { Some(s) } else { None }
            })
            .unwrap_or("overlay.jpg");

        let filename = read_line_trimmed(&format!("Save as filename [{default_fname}]: "))?;
        let filename = if filename.is_empty() {
            default_fname.to_string()
        } else {
            filename
        };

        let dir = overlays_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let output_path = dir.join(&filename);

        println!("Downloading...");
        let status = std::process::Command::new("curl")
            .args([
                "-L",
                "--fail",
                "--silent",
                "--show-error",
                "-o",
                &output_path.to_string_lossy(),
                &url,
            ])
            .status()?;

        if !status.success() {
            return Err(eyre!("Download failed (curl exit code: {status})"));
        }

        let size = fs::metadata(&output_path)?.len();
        println!("Downloaded {} ({size} bytes)", output_path.display());

        let stem = Path::new(&filename)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_uppercase()
            .replace(['-', '.', ' '], "_");
        let const_name = format!("{stem}_SPR");

        register_overlay_in_segments_rs(&const_name, &filename)?;
        println!("Registered as {const_name}.");

        Ok(const_name)
    } else {
        Ok(overlay_entries[idx].0.clone())
    }
}

fn insert_easter_egg_entry(entry: &str) -> color_eyre::Result<()> {
    let path = Path::new(PROJECT_DIR).join("src").join("eastereggs.rs");
    let content = fs::read_to_string(&path)?;

    let marker = "    })";
    let pos = content
        .rfind(marker)
        .ok_or_else(|| eyre!("Could not find easter_eggs! macro closing in eastereggs.rs"))?;

    let mut new_content = String::with_capacity(content.len() + entry.len() + 2);
    new_content.push_str(&content[..pos]);
    new_content.push('\n');
    new_content.push_str(entry);
    new_content.push('\n');
    new_content.push_str(&content[pos..]);

    fs::write(&path, new_content)?;
    Ok(())
}

impl YoMamaJoke {
    #[allow(dead_code)]
    pub fn record_and_save(output_path: &Path) -> color_eyre::Result<()> {
        let prompt = PolyPrompt::new(Backend::Bearask);

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| eyre!("no input device found"))?;

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let fmt = config.sample_format();
        let samples: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
        let stream = {
            let buf = Arc::clone(&samples);
            let err_fn = |e| eprintln!("Stream error: {e}");

            match fmt {
                SampleFormat::I16 => device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _| buf.lock().unwrap().extend_from_slice(data),
                    err_fn,
                    None,
                )?,
                SampleFormat::U16 => device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _| {
                        buf.lock()
                            .unwrap()
                            .extend(data.iter().map(|&s| s.wrapping_sub(32768) as i16));
                    },
                    err_fn,
                    None,
                )?,
                SampleFormat::F32 | _ => device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        buf.lock().unwrap().extend(
                            data.iter()
                                .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16),
                        );
                    },
                    err_fn,
                    None,
                )?,
            }
        };

        prompt
            .confirm("Ready to record? Press enter to start...")
            .with_default(true)
            .run()?;

        stream.play()?;

        prompt
            .confirm("Recording... Press enter to stop.")
            .with_default(true)
            .run()?;

        drop(stream);

        let samples = Arc::try_unwrap(samples)
            .expect("No other Arc owners")
            .into_inner()?;

        let mut encoder = Builder::new().expect("LAME init");
        encoder
            .set_num_channels(channels as u8)
            .expect("set channels");
        encoder
            .set_sample_rate(sample_rate)
            .expect("set sample rate");
        encoder.set_brate(Birtate::Kbps192).expect("set bitrate");
        encoder.set_quality(Quality::Best).expect("set quality");
        let mut encoder = encoder.build().expect("LAME build");
        let mut mp3_buf = Vec::new();
        let mp3_bytes = if channels == 1 {
            let pcm = MonoPcm(&samples);
            mp3_buf.reserve(mp3lame_encoder::max_required_buffer_size(samples.len()));
            let n = encoder
                .encode(pcm, mp3_buf.spare_capacity_mut())
                .expect("Failed");
            unsafe { mp3_buf.set_len(mp3_buf.len() + n) };
            let n = encoder
                .flush::<FlushNoGap>(mp3_buf.spare_capacity_mut())
                .expect("Failed");
            unsafe { mp3_buf.set_len(mp3_buf.len() + n) };
            mp3_buf
        } else {
            let left: Vec<i16> = samples.iter().copied().step_by(2).collect();
            let right: Vec<i16> = samples.iter().copied().skip(1).step_by(2).collect();
            let pcm = DualPcm {
                left: &left,
                right: &right,
            };
            mp3_buf.reserve(mp3lame_encoder::max_required_buffer_size(left.len()));
            let n = encoder
                .encode(pcm, mp3_buf.spare_capacity_mut())
                .expect("Failed");
            unsafe { mp3_buf.set_len(mp3_buf.len() + n) };
            let n = encoder
                .flush::<FlushNoGap>(mp3_buf.spare_capacity_mut())
                .expect("Failed");
            unsafe { mp3_buf.set_len(mp3_buf.len() + n) };
            mp3_buf
        };

        fs::write(output_path, &mp3_bytes)?;

        Ok(())
    }
}

impl YoMama {
    pub fn record_new() -> color_eyre::Result<()> {
        let prompt = PolyPrompt::new(Backend::Bearask);

        let file_count = |p: &Path| -> usize { fs::read_dir(p).map(|rd| rd.count()).unwrap_or(0) };

        let category = prompt
            .select::<String>("Which segment category?")
            .with_options(
                CATEGORIES
                    .iter()
                    .map(|&c| {
                        PolyOption::new(
                            format!(
                                "{} ({})",
                                c,
                                file_count(&Path::new(PROJECT_DIR).join("assets").join(c))
                            ),
                            c.to_string(),
                        )
                    })
                    .collect(),
            )
            .run()?;

        let category_dir = Path::new(PROJECT_DIR).join("assets").join(&category);

        if !category_dir.exists() {
            fs::create_dir_all(&category_dir)?;
        }

        let use_custom = prompt
            .confirm("Use a custom filename? (No = auto-numbered)")
            .with_default(false)
            .run()?;

        let output_path = if use_custom {
            let name = read_line_trimmed("Enter filename (without .mp3): ")?;
            if name.is_empty() {
                return Err(eyre!("No filename provided"));
            }
            category_dir.join(format!("{name}.mp3"))
        } else {
            next_filename(&category_dir)
        };

        println!("Recording to: {}", output_path.display());
        YoMamaJoke::record_and_save(&output_path)?;
        println!("Saved! Rebuild to include the new segment (cargo build).");

        Ok(())
    }
}
