use std::{env, fs, path::Path};

fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets_dir = Path::new(&manifest_dir).join("assets");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("segments.rs");

    println!("cargo::rerun-if-changed=assets");

    let mut out = String::new();

    let mut categories: Vec<_> = fs::read_dir(&assets_dir)
        .expect("assets/ directory not found — create it next to Cargo.toml")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| e.file_name().to_str().unwrap() != "overlays")
        .collect();
    categories.sort_by_key(|e| e.file_name());

    for category_entry in categories {
        let category = category_entry.file_name().to_string_lossy().into_owned();
        let mod_name = category.replace('-', "_");
        let prefix = mod_name.to_uppercase();

        let mut files: Vec<_> = fs::read_dir(category_entry.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("mp3"))
                    .unwrap_or(false)
            })
            .collect();
        files.sort_by_key(|e| e.file_name());

        out.push_str(&format!("pub mod {mod_name} {{\n"));
        out.push_str("    use crate::SegmentClip;\n");

        let mut const_names: Vec<String> = Vec::new();

        for file in &files {
            let fname = file.file_name().to_string_lossy().into_owned();
            let file_contents = fs::read(file.path()).unwrap();
            let hash = fnv1a(&file_contents);

            let stem = Path::new(&fname)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_uppercase()
                .replace(['-', '.'], "_");

            let const_name = format!("{prefix}_{stem}");

            out.push_str(&format!(
                "    pub static {const_name}: SegmentClip = SegmentClip {{\n\
                 \x20       data: include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \
                 \"/assets/{category}/{fname}\")),\n\
                 \x20       hash: 0x{hash:016x}_u64,\n\
                 \x20       name: \"{fname}\",\n\
                 \x20   }};\n"
            ));

            const_names.push(const_name);
        }

        let all_items = const_names
            .iter()
            .map(|n| format!("&{n}"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "    pub static ALL: &[&SegmentClip] = &[{all_items}];\n"
        ));
        out.push_str("}\n\n");
    }

    fs::write(&dest, &out).expect("Failed to write generated segments.rs");
}
