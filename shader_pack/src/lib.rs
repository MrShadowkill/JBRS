use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

const MAX_SOURCE_BYTES: u64 = 1024 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 4096;
const MAX_DIRECTORY_DEPTH: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderPair {
    pub vertex: String,
    pub fragment: String,
    pub source: String,
}

/// Directories where user-installed shader packs live. Pack names are plain
/// directory or ZIP stems, so the returned IDs are safe to persist in saves.
pub fn available_packs(platform: &str) -> Vec<String> {
    let mut packs = vec!["builtin".to_owned()];
    for root in pack_roots() {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !valid_pack_id(name) {
                continue;
            }
            let found = if path.is_dir() {
                find_pack_directory(&path, platform)
                    .ok()
                    .flatten()
                    .is_some()
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
            {
                read_archive(&path)
                    .ok()
                    .and_then(|bytes| load_archive(&bytes, platform, name).ok().flatten())
                    .is_some()
            } else {
                false
            };
            if found && !packs.iter().any(|pack| pack.eq_ignore_ascii_case(name)) {
                packs.push(name.to_owned());
            }
        }
    }
    packs.sort_by_key(|name| name.to_ascii_lowercase());
    packs.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    if let Some(index) = packs.iter().position(|name| name == "builtin") {
        packs.swap(0, index);
    }
    packs
}

pub fn load_pack(id: &str, platform: &str) -> Result<Option<ShaderPair>, String> {
    if id.eq_ignore_ascii_case("builtin") {
        return Ok(None);
    }
    if !valid_pack_id(id) {
        return Err("invalid shader pack name".into());
    }
    for root in pack_roots() {
        let directory = root.join(id);
        if directory.is_dir() {
            if let Some(pair) = find_pack_directory(&directory, platform)? {
                return Ok(Some(pair));
            }
        }
        let archive = root.join(format!("{id}.zip"));
        if archive.is_file() {
            let bytes = read_archive(&archive)?;
            return load_archive(&bytes, platform, &archive.display().to_string());
        }
    }
    Err(format!("shader pack '{id}' was not found for {platform}"))
}

fn valid_pack_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn pack_roots() -> Vec<PathBuf> {
    let mut roots = roots()
        .into_iter()
        .map(|root| root.join("packs"))
        .collect::<Vec<_>>();
    if let Some(path) = std::env::var_os("JBRS_SHADER_DIR") {
        roots.push(PathBuf::from(path).join("packs"));
    }
    #[cfg(windows)]
    {
        let root = PathBuf::from(r"C:\JBRS\shaders\packs");
        let _ = fs::create_dir_all(&root);
        roots.push(root);
    }
    #[cfg(target_os = "android")]
    {
        let root = PathBuf::from("/storage/emulated/0/JBRS/shaders/packs");
        let _ = fs::create_dir_all(&root);
        roots.push(root);
    }
    #[cfg(not(any(windows, target_os = "android")))]
    if let Some(home) = std::env::var_os("HOME") {
        let root = PathBuf::from(home).join("JBRS/shaders/packs");
        let _ = fs::create_dir_all(&root);
        roots.push(root);
    }
    let mut unique = Vec::new();
    for root in roots {
        if !unique.contains(&root) {
            unique.push(root);
        }
    }
    unique
}

fn find_pack_directory(root: &Path, platform: &str) -> Result<Option<ShaderPair>, String> {
    let platform_root = root.join(platform);
    if platform_root.is_dir() {
        return find_directory_pair(&platform_root, platform);
    }
    find_directory_pair(root, platform)
}

pub fn from_sources(
    vertex: &[u8],
    fragment: &[u8],
    source: impl Into<String>,
) -> Option<ShaderPair> {
    if vertex.len() as u64 > MAX_SOURCE_BYTES || fragment.len() as u64 > MAX_SOURCE_BYTES {
        return None;
    }
    Some(ShaderPair {
        vertex: String::from_utf8(vertex.to_vec()).ok()?,
        fragment: String::from_utf8(fragment.to_vec()).ok()?,
        source: source.into(),
    })
}

/// Load the first matching shader pair from the configured override directory or
/// from a `shaders` directory next to the executable. ZIPs are read in place and
/// are never extracted to disk.
pub fn load_external(platform: &str) -> Result<Option<ShaderPair>, String> {
    for root in roots() {
        if !root.is_dir() {
            continue;
        }
        let platform_root = root.join(platform);
        if platform_root.is_dir() {
            if let Some(pair) = find_directory_pair(&platform_root, platform)? {
                return Ok(Some(pair));
            }
        }
        if let Some(pair) = find_directory_pair(&root, platform)? {
            return Ok(Some(pair));
        }
    }
    Ok(None)
}

pub fn load_archive(
    bytes: &[u8],
    platform: &str,
    source: &str,
) -> Result<Option<ShaderPair>, String> {
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(format!("shader archive {source} exceeds the 64 MiB limit"));
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("cannot open shader archive {source}: {error}"))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!("shader archive {source} has too many entries"));
    }
    let mut folders: BTreeMap<String, BTreeMap<String, Vec<u8>>> = BTreeMap::new();
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("cannot read shader archive {source}: {error}"))?;
        if file.is_dir() || file.size() > MAX_SOURCE_BYTES {
            continue;
        }
        let name = file.name().replace('\\', "/");
        let (folder, basename) = name.rsplit_once('/').unwrap_or(("", name.as_str()));
        let mut data = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut data)
            .map_err(|error| format!("cannot read {name} in {source}: {error}"))?;
        folders
            .entry(folder.to_owned())
            .or_default()
            .insert(basename.to_ascii_lowercase(), data);
    }
    let platform = platform.to_ascii_lowercase();
    let mut candidates = Vec::new();
    for (folder, files) in folders {
        if let Some((vertex_name, fragment_name)) = select_pair(files.keys().map(String::as_str)) {
            let platform_match = folder
                .split('/')
                .any(|part| part.eq_ignore_ascii_case(&platform));
            let (Some(vertex), Some(fragment)) = (files.get(vertex_name), files.get(fragment_name))
            else {
                continue;
            };
            if let Some(pair) = from_sources(
                vertex,
                fragment,
                format!("{source}/{folder}/{vertex_name} + {fragment_name}"),
            ) {
                candidates.push((!platform_match, folder, pair));
            }
        }
    }
    candidates.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    Ok(candidates.into_iter().next().map(|(_, _, pair)| pair))
}

fn roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(path) = std::env::var_os("JBRS_SHADER_DIR") {
        roots.push(PathBuf::from(path));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            roots.push(parent.join("shaders"));
            if let Some(grandparent) = parent.parent() {
                roots.push(grandparent.join("shaders"));
            }
        }
    }
    if let Ok(current) = std::env::current_dir() {
        roots.push(current.join("shaders"));
    }
    let mut unique = Vec::new();
    for root in roots {
        if !unique.contains(&root) {
            unique.push(root);
        }
    }
    unique
}

fn find_directory_pair(root: &Path, platform: &str) -> Result<Option<ShaderPair>, String> {
    let mut folders = Vec::new();
    collect_directories(root, 0, platform, &mut folders);
    folders.sort();
    for folder in folders {
        let Ok(entries) = fs::read_dir(&folder) else {
            continue;
        };
        let files: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .collect();
        let names: Vec<_> = files
            .iter()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        if let Some((vertex, fragment)) = select_pair(names.iter().map(String::as_str)) {
            let vertex_path = folder.join(vertex);
            let fragment_path = folder.join(fragment);
            let vertex_source = read_limited(&vertex_path)?;
            let fragment_source = read_limited(&fragment_path)?;
            if let Some(pair) = from_sources(
                &vertex_source,
                &fragment_source,
                format!("{} + {}", vertex_path.display(), fragment_path.display()),
            ) {
                return Ok(Some(pair));
            }
            return Err(format!(
                "shader pair in {} must be UTF-8 text no larger than 1 MiB per file",
                folder.display()
            ));
        }
        let mut archives: Vec<_> = files
            .iter()
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
            })
            .map(|entry| entry.path())
            .collect();
        archives.sort();
        for archive_path in archives {
            let bytes = read_archive(&archive_path)?;
            let Some(filename) = archive_path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if let Some(pair) = load_archive(&bytes, platform, filename)? {
                return Ok(Some(pair));
            }
        }
    }
    Ok(None)
}

fn collect_directories(path: &Path, depth: usize, platform: &str, output: &mut Vec<PathBuf>) {
    if depth > MAX_DIRECTORY_DEPTH {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    output.push(path.to_path_buf());
    for entry in entries.filter_map(Result::ok) {
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            let skip_platform = entry.file_name().to_str().is_some_and(|name| {
                (name.eq_ignore_ascii_case("desktop") && platform != "desktop")
                    || (name.eq_ignore_ascii_case("gles2") && platform != "gles2")
            });
            if skip_platform {
                continue;
            }
            collect_directories(&entry.path(), depth + 1, platform, output);
        }
    }
}

fn read_limited(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(format!("{} exceeds the 1 MiB shader limit", path.display()));
    }
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn read_archive(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.len() > MAX_ARCHIVE_BYTES {
        return Err(format!(
            "{} exceeds the 64 MiB archive limit",
            path.display()
        ));
    }
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

const MAX_TEXTURE_PACK_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TEXTURE_PNG_BYTES: u64 = 8 * 1024 * 1024;
const TEXTURE_FILES: [(&str, usize); 6] = [
    ("roads.png", 0),
    ("floors.png", 1),
    ("walls.png", 12),
    ("windows.png", 3),
    ("metal.png", 4),
    ("roofs.png", 11),
];

/// Texture pack directories and ZIP files are discovered in `texture-packs`
/// roots next to the executable, in the repository assets directory, or in the
/// user's JBRS data directory. The ID is the directory/archive stem.
pub fn available_texture_packs() -> Vec<String> {
    static PACKS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    PACKS.get_or_init(discover_texture_packs).clone()
}

fn discover_texture_packs() -> Vec<String> {
    let mut packs = vec!["builtin".to_owned()];
    for root in texture_pack_roots() {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !valid_pack_id(id) || id.eq_ignore_ascii_case("builtin") {
                continue;
            }
            let found = if path.is_dir() {
                directory_texture_files(&path).is_ok_and(|files| !files.is_empty())
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
            {
                archive_has_textures(&path).unwrap_or(false)
            } else {
                false
            };
            if found && !packs.iter().any(|known| known.eq_ignore_ascii_case(id)) {
                packs.push(id.to_owned());
            }
        }
    }
    packs.sort_by_key(|name| name.to_ascii_lowercase());
    packs.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    if let Some(index) = packs.iter().position(|name| name == "builtin") {
        packs.swap(0, index);
    }
    packs
}

/// Replace available atlas tiles with PNGs from a selected texture pack.
/// Atlas construction and image decoding happen on startup or when the user
/// changes pack/texture quality, never in the draw loop.
pub fn load_texture_atlas(id: &str, tile: usize, fallback: &[u8]) -> Result<Vec<u8>, String> {
    if !(16..=512).contains(&tile) {
        return Err("invalid texture atlas dimensions".into());
    }
    let expected = (tile * 4) * (tile * 4) * 3;
    if fallback.len() != expected {
        return Err("invalid texture atlas dimensions".into());
    }
    if id.eq_ignore_ascii_case("builtin") {
        return Ok(fallback.to_vec());
    }
    if !valid_pack_id(id) {
        return Err("invalid texture pack name".into());
    }
    let mut sources = None;
    for root in texture_pack_roots() {
        let directory = root.join(id);
        if directory.is_dir() {
            sources = Some(directory_texture_files(&directory)?);
            if sources.as_ref().is_some_and(|files| !files.is_empty()) {
                break;
            }
        }
        let archive = root.join(format!("{id}.zip"));
        if archive.is_file() {
            sources = Some(read_texture_archive(&archive)?);
            if sources.as_ref().is_some_and(|files| !files.is_empty()) {
                break;
            }
        }
    }
    let Some(sources) = sources.filter(|files| !files.is_empty()) else {
        return Err(format!(
            "texture pack '{id}' was not found or has no supported PNGs"
        ));
    };
    apply_texture_sources(fallback, tile, &sources)
}

/// Assemble an atlas from PNG bytes supplied by a platform asset provider.
/// This is used by Android for packs bundled in the APK asset directory.
pub fn texture_atlas_from_files(
    tile: usize,
    fallback: &[u8],
    files: impl IntoIterator<Item = (String, Vec<u8>)>,
) -> Result<Vec<u8>, String> {
    if !(16..=512).contains(&tile) {
        return Err("invalid texture atlas dimensions".into());
    }
    let expected = (tile * 4) * (tile * 4) * 3;
    if fallback.len() != expected {
        return Err("invalid texture atlas dimensions".into());
    }
    let sources = files
        .into_iter()
        .map(|(name, bytes)| (name.to_ascii_lowercase(), bytes))
        .collect();
    apply_texture_sources(fallback, tile, &sources)
}

fn apply_texture_sources(
    fallback: &[u8],
    tile: usize,
    sources: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let mut atlas = fallback.to_vec();
    let atlas_width = tile * 4;
    for (filename, material) in TEXTURE_FILES {
        let Some(bytes) = sources.get(filename) else {
            continue;
        };
        let Some(rgb) = decode_png(bytes, filename)? else {
            continue;
        };
        let x0 = material % 4 * tile;
        let y0 = material / 4 * tile;
        for y in 0..tile {
            for x in 0..tile {
                let source_x = x * rgb.width / tile;
                let source_y = y * rgb.height / tile;
                let source = (source_y * rgb.width + source_x) * 3;
                let dest = ((y0 + y) * atlas_width + x0 + x) * 3;
                atlas[dest..dest + 3].copy_from_slice(&rgb.pixels[source..source + 3]);
            }
        }
    }
    Ok(atlas)
}

struct RgbImage {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

fn decode_png(bytes: &[u8], name: &str) -> Result<Option<RgbImage>, String> {
    if bytes.len() as u64 > MAX_TEXTURE_PNG_BYTES {
        return Err(format!("texture {name} exceeds the 8 MiB limit"));
    }
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("cannot decode {name}: {error}"))?;
    let width = reader.info().width as usize;
    let height = reader.info().height as usize;
    if width == 0 || height == 0 || width > 2048 || height > 2048 {
        return Err(format!("texture {name} dimensions must be 1..=2048 pixels"));
    }
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| format!("cannot read PNG pixels from {name}: {error}"))?;
    let pixels = &buffer[..info.buffer_size()];
    let mut rgb = Vec::with_capacity(width * height * 3);
    match info.color_type {
        png::ColorType::Rgb => rgb.extend_from_slice(pixels),
        png::ColorType::Rgba => {
            for rgba in pixels.chunks_exact(4) {
                rgb.extend_from_slice(&rgba[..3]);
            }
        }
        png::ColorType::Grayscale => {
            for value in pixels {
                rgb.extend_from_slice(&[*value, *value, *value]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for gray_alpha in pixels.chunks_exact(2) {
                rgb.extend_from_slice(&[gray_alpha[0]; 3]);
            }
        }
        png::ColorType::Indexed => {
            return Err(format!(
                "indexed PNG {name} was not expanded by the decoder"
            ));
        }
    }
    Ok(Some(RgbImage {
        width,
        height,
        pixels: rgb,
    }))
}

fn texture_pack_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(path) = std::env::var_os("JBRS_TEXTURE_DIR") {
        let path = PathBuf::from(path);
        let _ = fs::create_dir_all(&path);
        roots.push(path);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            roots.push(parent.join("texture-packs"));
            roots.push(parent.join("assets/texture-packs"));
            if let Some(grandparent) = parent.parent() {
                roots.push(grandparent.join("assets/texture-packs"));
            }
        }
    }
    if let Ok(current) = std::env::current_dir() {
        roots.push(current.join("texture-packs"));
        roots.push(current.join("assets/texture-packs"));
    }
    #[cfg(windows)]
    {
        let root = PathBuf::from(r"C:\JBRS\texture-packs");
        let _ = fs::create_dir_all(&root);
        roots.push(root);
    }
    #[cfg(target_os = "android")]
    {
        let root = PathBuf::from("/storage/emulated/0/JBRS/texture-packs");
        let _ = fs::create_dir_all(&root);
        roots.push(root);
    }
    #[cfg(not(any(windows, target_os = "android")))]
    if let Some(home) = std::env::var_os("HOME") {
        let root = PathBuf::from(home).join("JBRS/texture-packs");
        let _ = fs::create_dir_all(&root);
        roots.push(root);
    }
    let mut unique = Vec::new();
    for root in roots {
        if !unique.contains(&root) {
            unique.push(root);
        }
    }
    unique
}

fn directory_texture_files(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut files = BTreeMap::new();
    let paths = [
        root.join("assets/textures"),
        root.join("textures"),
        root.to_path_buf(),
    ];
    for folder in paths {
        for (name, _) in TEXTURE_FILES {
            let path = folder.join(name);
            if !path.is_file() || files.contains_key(name) {
                continue;
            }
            let metadata = fs::metadata(&path)
                .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
            if metadata.len() > MAX_TEXTURE_PNG_BYTES {
                return Err(format!(
                    "texture {} exceeds the 8 MiB limit",
                    path.display()
                ));
            }
            files.insert(
                name.to_owned(),
                fs::read(&path)
                    .map_err(|error| format!("cannot read texture {}: {error}", path.display()))?,
            );
        }
    }
    Ok(files)
}

fn archive_texture_names(path: &Path) -> Result<Vec<(usize, String)>, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("cannot inspect texture pack {}: {error}", path.display()))?;
    if metadata.len() > MAX_TEXTURE_PACK_ARCHIVE_BYTES {
        return Err(format!("texture pack {} exceeds 64 MiB", path.display()));
    }
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("cannot open texture pack {}: {error}", path.display()))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!(
            "texture pack {} has too many entries",
            path.display()
        ));
    }
    let mut found = Vec::new();
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|error| format!("cannot inspect texture pack: {error}"))?;
        if file.is_dir() || file.size() > MAX_TEXTURE_PNG_BYTES {
            continue;
        }
        let name = file.name().replace('\\', "/");
        let normalized = name.to_ascii_lowercase();
        if let Some((filename, _)) = TEXTURE_FILES.iter().find(|(filename, _)| {
            normalized == *filename || normalized.ends_with(&format!("/textures/{filename}"))
        }) {
            found.push((index, (*filename).to_owned()));
        }
    }
    Ok(found)
}

fn archive_has_textures(path: &Path) -> Result<bool, String> {
    Ok(!archive_texture_names(path)?.is_empty())
}

fn read_texture_archive(path: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("cannot inspect texture pack {}: {error}", path.display()))?;
    if metadata.len() > MAX_TEXTURE_PACK_ARCHIVE_BYTES {
        return Err(format!("texture pack {} exceeds 64 MiB", path.display()));
    }
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("cannot open texture pack {}: {error}", path.display()))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!(
            "texture pack {} has too many entries",
            path.display()
        ));
    }
    let mut files = BTreeMap::new();
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("cannot read texture pack: {error}"))?;
        if file.is_dir() || file.size() > MAX_TEXTURE_PNG_BYTES {
            continue;
        }
        let name = file.name().replace('\\', "/").to_ascii_lowercase();
        let Some((filename, _)) = TEXTURE_FILES.iter().find(|(filename, _)| {
            name == *filename || name.ends_with(&format!("/textures/{filename}"))
        }) else {
            continue;
        };
        let mut data = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut data)
            .map_err(|error| format!("cannot read {name} from {}: {error}", path.display()))?;
        files.insert((*filename).to_owned(), data);
    }
    Ok(files)
}

fn select_pair<'a>(names: impl Iterator<Item = &'a str>) -> Option<(&'a str, &'a str)> {
    let names: Vec<_> = names.collect();
    let mut pairs = Vec::new();
    for vertex in &names {
        let vertex_lower = vertex.to_ascii_lowercase();
        let Some(vertex_stem) = shader_stem(&vertex_lower, true) else {
            continue;
        };
        for fragment in &names {
            let fragment_lower = fragment.to_ascii_lowercase();
            let Some(fragment_stem) = shader_stem(&fragment_lower, false) else {
                continue;
            };
            if vertex_stem == fragment_stem {
                pairs.push((
                    vertex_stem != "world",
                    vertex_stem.to_owned(),
                    *vertex,
                    *fragment,
                ));
            }
        }
    }
    pairs.sort_by(|a, b| (&a.0, &a.1, a.2, a.3).cmp(&(&b.0, &b.1, b.2, b.3)));
    if let Some((_, _, vertex, fragment)) = pairs.first() {
        return Some((vertex, fragment));
    }

    let mut vertex = Vec::new();
    let mut fragment = Vec::new();
    for name in names {
        let lower = name.to_ascii_lowercase();
        if is_vertex(&lower) {
            vertex.push(name);
        }
        if is_fragment(&lower) {
            fragment.push(name);
        }
    }
    vertex.sort_by_key(|name| shader_priority(name, true));
    fragment.sort_by_key(|name| shader_priority(name, false));
    let v = *vertex.first()?;
    let f = *fragment.first()?;
    Some((v, f))
}

fn shader_stem(name: &str, vertex: bool) -> Option<&str> {
    let suffixes: &[&str] = if vertex {
        &[".vert.glsl", ".vertex", ".vert", ".vsh", ".vs"]
    } else {
        &[".frag.glsl", ".fragment", ".frag", ".fsh", ".fs"]
    };
    suffixes.iter().find_map(|suffix| name.strip_suffix(suffix))
}

fn is_vertex(name: &str) -> bool {
    [".vert", ".vert.glsl", ".vertex", ".vs", ".vsh"]
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn is_fragment(name: &str) -> bool {
    [".frag", ".frag.glsl", ".fragment", ".fs", ".fsh"]
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn shader_priority(name: &str, vertex: bool) -> u8 {
    let name = name.to_ascii_lowercase();
    let canonical = if vertex {
        [
            "world.vert",
            "world.vert.glsl",
            "vertex.vert",
            "vertex.glsl",
        ]
    } else {
        [
            "world.frag",
            "world.frag.glsl",
            "fragment.frag",
            "fragment.glsl",
        ]
    };
    canonical
        .iter()
        .position(|candidate| *candidate == name)
        .map(|priority| priority as u8)
        .unwrap_or(10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_any_vertex_and_fragment_extension_from_a_folder() {
        let temp = std::env::temp_dir().join(format!("jbrs-shaders-{}", std::process::id()));
        let folder = temp.join("desktop");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("main.vsh"), "vertex").unwrap();
        fs::write(folder.join("main.fsh"), "fragment").unwrap();
        let pair = find_directory_pair(&temp, "desktop").unwrap().unwrap();
        assert_eq!(pair.vertex, "vertex");
        assert_eq!(pair.fragment, "fragment");
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn pairs_vertex_and_fragment_by_filename_stem() {
        let pair = select_pair(["vehicle.vert", "world.frag", "vehicle.frag"].into_iter()).unwrap();
        assert_eq!(pair, ("vehicle.vert", "vehicle.frag"));
    }

    #[test]
    fn reads_shader_pair_from_a_zip_without_extracting_it() {
        let mut bytes = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut bytes);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("some-pack/gles2/world.vert", options)
                .unwrap();
            zip.write_all(b"gles vertex").unwrap();
            zip.start_file("some-pack/gles2/world.frag", options)
                .unwrap();
            zip.write_all(b"gles fragment").unwrap();
            zip.start_file("some-pack/desktop/world.vert", options)
                .unwrap();
            zip.write_all(b"gl vertex").unwrap();
            zip.start_file("some-pack/desktop/world.frag", options)
                .unwrap();
            zip.write_all(b"gl fragment").unwrap();
            zip.finish().unwrap();
        }
        let pair = load_archive(bytes.get_ref(), "gles2", "test.zip")
            .unwrap()
            .unwrap();
        assert_eq!(pair.vertex, "gles vertex");
        assert_eq!(pair.fragment, "gles fragment");
    }

    #[test]
    fn sample_texture_pack_pngs_replace_wall_window_and_road_atlas_tiles() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/texture-packs/C13HPT-Template");
        let files = directory_texture_files(&root).unwrap();
        assert_eq!(files.len(), 6);
        let tile = 16usize;
        let fallback = vec![17; (tile * 4).pow(2) * 3];
        let atlas = apply_texture_sources(&fallback, tile, &files).unwrap();
        let platform_atlas = texture_atlas_from_files(tile, &fallback, files.clone()).unwrap();
        assert_eq!(platform_atlas, atlas);
        let tile_pixels = |material: usize, pixels: &[u8]| {
            let mut result = Vec::with_capacity(tile * tile * 3);
            for y in 0..tile {
                let offset = ((material / 4 * tile + y) * tile * 4 + material % 4 * tile) * 3;
                result.extend_from_slice(&pixels[offset..offset + tile * 3]);
            }
            result
        };
        for material in [0, 1, 3, 4, 11, 12] {
            assert_ne!(
                tile_pixels(material, &atlas),
                tile_pixels(material, &fallback)
            );
        }
        assert_eq!(tile_pixels(2, &atlas), tile_pixels(2, &fallback));
    }

    #[test]
    fn texture_pack_zip_reads_nested_assets_without_extracting() {
        let png = include_bytes!(
            "../../assets/texture-packs/C13HPT-Template/assets/textures/windows.png"
        );
        let path = std::env::temp_dir().join(format!("jbrs-textures-{}.zip", std::process::id()));
        {
            let file = fs::File::create(&path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            zip.start_file(
                "Example Pack/assets/textures/windows.png",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(png).unwrap();
            zip.finish().unwrap();
        }
        let files = read_texture_archive(&path).unwrap();
        fs::remove_file(path).unwrap();
        assert!(files.contains_key("windows.png"));
        let tile = 16usize;
        let fallback = vec![17; (tile * 4).pow(2) * 3];
        let atlas = apply_texture_sources(&fallback, tile, &files).unwrap();
        let index = (tile * 8 + tile * 3) * 3;
        assert_ne!(&atlas[index..index + 3], &fallback[index..index + 3]);
    }
}
