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
}
