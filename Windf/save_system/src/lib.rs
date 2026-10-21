use engine_core::{Team, Vec3, VEHICLE_SLOTS};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};
#[derive(Serialize, Deserialize)]
pub struct Save {
    #[serde(default)]
    pub searched_furniture: Vec<u64>,
    #[serde(default)]
    pub heist_cooldown: [f32; 6],
    #[serde(default)]
    pub robbery_cooldown: [f32; 7],
    #[serde(default)]
    pub wanted: u32,
    #[serde(default)]
    pub bounty: u32,
    #[serde(default)]
    #[serde(deserialize_with = "read_car_engine")]
    pub car_engine: [u32; VEHICLE_SLOTS],
    #[serde(default = "default_selected_engine")]
    #[serde(deserialize_with = "read_car_engine_selected")]
    pub car_engine_selected: [u32; VEHICLE_SLOTS],
    #[serde(default)]
    #[serde(deserialize_with = "read_car_brakes")]
    pub car_brakes: [u32; VEHICLE_SLOTS],
    #[serde(default)]
    #[serde(deserialize_with = "read_car_suspension")]
    pub car_suspension: [u32; VEHICLE_SLOTS],
    #[serde(default)]
    #[serde(deserialize_with = "read_car_paint")]
    pub car_paint: [[f32; 3]; VEHICLE_SLOTS],
    #[serde(default)]
    pub weapon_owned: [bool; 3],
    #[serde(default)]
    pub magazines: [u32; 3],
    #[serde(default)]
    pub ammunition: [u32; 3],
    #[serde(default)]
    pub leases: [f32; 4],
    #[serde(default)]
    pub texture_level: usize,
    #[serde(default = "default_shadows")]
    pub object_shadows: bool,
    #[serde(default)]
    pub run_level: u32,
    #[serde(default)]
    pub endurance_level: u32,
    #[serde(default)]
    pub carry_level: u32,
    #[serde(default)]
    pub water: u32,
    #[serde(default)]
    pub food: u32,
    #[serde(default)]
    pub pistol_owned: bool,
    #[serde(default = "default_quality")]
    pub quality: usize,
    #[serde(default = "default_shader_pack")]
    pub shader_pack: String,
    #[serde(default)]
    pub auto_shader_pack: bool,
    #[serde(default = "default_resolution")]
    pub resolution_scale: u8,
    #[serde(default)]
    pub fps_counter: bool,
    #[serde(default = "default_shader_effects")]
    pub shader_effects: u8,
    #[serde(default = "default_shadow_quality")]
    pub shadow_quality: u8,
    pub version: u32,
    pub balance: u32,
    pub owned: Vec<usize>,
    pub color: [f32; 3],
    pub position: Vec3,
    #[serde(deserialize_with = "read_team")]
    pub team: Team,
    pub robberies: u32,
    pub sensitivity: f32,
    #[serde(default = "default_ammo")]
    pub ammo: u32,
    #[serde(default = "default_reserve")]
    pub reserve: u32,
    #[serde(default = "default_distance")]
    pub render_distance: f32,
}
impl Default for Save {
    fn default() -> Self {
        Self {
            searched_furniture: Vec::new(),
            heist_cooldown: [0.; 6],
            robbery_cooldown: [0.; 7],
            wanted: 0,
            bounty: 0,
            car_engine: [0; VEHICLE_SLOTS],
            car_engine_selected: [u32::MAX; VEHICLE_SLOTS],
            car_brakes: [0; VEHICLE_SLOTS],
            car_suspension: [0; VEHICLE_SLOTS],
            car_paint: [[0.; 3]; VEHICLE_SLOTS],
            weapon_owned: [false; 3],
            magazines: [0; 3],
            ammunition: [0; 3],
            leases: [0.; 4],
            texture_level: 0,
            object_shadows: true,
            run_level: 0,
            endurance_level: 0,
            carry_level: 0,
            water: 0,
            food: 0,
            pistol_owned: false,
            quality: 1,
            shader_pack: "builtin".into(),
            auto_shader_pack: false,
            resolution_scale: 100,
            fps_counter: false,
            shader_effects: 4,
            shadow_quality: 1,
            version: 1,
            balance: 0,
            owned: vec![],
            color: [0.8, 0.24, 0.12],
            position: Vec3::new(-100., 1., -160.),
            team: Team::Prisoner,
            robberies: 0,
            sensitivity: 0.003,
            ammo: 0,
            reserve: 0,
            render_distance: 450.,
        }
    }
}
pub fn path() -> PathBuf {
    if let Some(p) = std::env::var_os("JBRS_SAVE") {
        return p.into();
    }
    #[cfg(windows)]
    {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("AppData/Local"))
            })
            .unwrap_or_else(std::env::temp_dir);
        base.join("JBRS-Windf/save.json")
    }
    #[cfg(not(windows))]
    {
        let mut p = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
            });
        p.push("jbrs/save.json");
        p
    }
}
pub fn load() -> Result<Save, String> {
    let current = path();
    if !cfg!(windows) && !current.exists() && std::env::var_os("JBRS_SAVE").is_none() {
        // Read the previous on-disk identifier once; subsequent saves use JBRS.
        if let Some(base) = current.parent().and_then(|p| p.parent()) {
            let legacy = base.join("jailbreak-rs/save.json");
            if legacy.is_file() {
                return load_recovering(&legacy);
            }
        }
    }
    load_recovering(&current)
}
/// Keep the exact original bytes before recovering any invalid document.
fn backup_invalid(p: &std::path::Path, bytes: &[u8]) -> Result<PathBuf, String> {
    use std::io::Write;
    for number in 1..=1000 {
        let backup = p.with_extension(format!("recovery-{number}.json"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup)
        {
            Ok(mut file) => {
                file.write_all(bytes)
                    .and_then(|()| file.sync_all())
                    .map_err(|e| {
                        format!(
                            "Cannot preserve save at {}: {e}. Original left untouched.",
                            backup.display()
                        )
                    })?;
                return Ok(backup);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(format!(
                    "Cannot back up {}: {e}. Original left untouched.",
                    p.display()
                ))
            }
        }
    }
    Err(format!(
        "Too many recovery backups for {}; original left untouched",
        p.display()
    ))
}
fn load_recovering(p: &std::path::Path) -> Result<Save, String> {
    // I/O failures are not evidence of corruption and must never trigger a reset.
    let bytes = match std::fs::read(p) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Save::default()),
        Err(e) => return Err(format!("Cannot read save {}: {e}", p.display())),
    };
    match decode(&bytes) {
        Ok(save) => Ok(save),
        Err(error) => {
            let backup = backup_invalid(p, &bytes)?;
            let (save, message) = match serde_json::from_slice::<Save>(&bytes) {
                Ok(mut save) if save.version == 1 => {
                    repair(&mut save);
                    // Recheck every invariant before returning the repaired state.
                    let encoded = serde_json::to_vec(&save).map_err(|e| e.to_string())?;
                    (
                        decode(&encoded)?,
                        "Recovered compatible progress and reset invalid values",
                    )
                }
                _ => (
                    Save::default(),
                    "Starting a fresh session; the original save is preserved",
                ),
            };
            eprintln!(
                "JBRS: save {} could not be loaded: {error}. {message}. Backup: {}",
                p.display(),
                backup.display()
            );
            Ok(save)
        }
    }
}
fn repair(s: &mut Save) {
    fn bounded(v: &mut f32, low: f32, high: f32, fallback: f32) {
        *v = if v.is_finite() {
            v.clamp(low, high)
        } else {
            fallback
        };
    }
    for v in s
        .heist_cooldown
        .iter_mut()
        .chain(s.robbery_cooldown.iter_mut())
        .chain(s.leases.iter_mut())
    {
        bounded(v, 0., 900., 0.);
    }
    s.wanted = s.wanted.min(5);
    for (selected, maximum) in s
        .car_engine_selected
        .iter_mut()
        .zip(s.car_engine.iter_mut())
    {
        *maximum = (*maximum).min(5);
        if *selected != u32::MAX {
            *selected = (*selected).min(*maximum);
        }
    }
    for v in &mut s.car_brakes {
        *v = (*v).min(3);
    }
    for v in &mut s.car_suspension {
        *v = (*v).min(2);
    }
    for v in s.car_paint.iter_mut().flatten().chain(s.color.iter_mut()) {
        bounded(v, 0., 1., 0.);
    }
    for (v, maximum) in s.magazines.iter_mut().zip([12, 6, 24]) {
        *v = (*v).min(maximum);
    }
    for v in &mut s.ammunition {
        *v = (*v).min(1000);
    }
    s.texture_level = s.texture_level.min(3);
    s.quality = s.quality.min(3);
    s.run_level = s.run_level.min(5);
    s.endurance_level = s.endurance_level.min(5);
    s.carry_level = s.carry_level.min(5);
    s.water = s.water.min(20);
    s.food = s.food.min(20);
    s.resolution_scale = s.resolution_scale.clamp(50, 100);
    s.shader_effects = s.shader_effects.min(4);
    s.shadow_quality = s.shadow_quality.min(3);
    s.owned.retain(|i| *i < VEHICLE_SLOTS);
    if !s.position.x.is_finite() || !s.position.y.is_finite() || !s.position.z.is_finite() {
        s.position = Save::default().position;
    }
    bounded(&mut s.render_distance, 80., 650., 450.);
    s.ammo = s.ammo.min(12);
    if !s.sensitivity.is_finite() || s.sensitivity <= 0. {
        s.sensitivity = 0.003;
    }
}
#[cfg(test)]
fn load_from(p: &std::path::Path) -> Result<Save, String> {
    if !p.exists() {
        return Ok(Save::default());
    }
    decode(&std::fs::read(p).map_err(|e| e.to_string())?)
}
fn decode(bytes: &[u8]) -> Result<Save, String> {
    let s: Save = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if s.version != 1 {
        return Err("Unsupported save version".into());
    }
    if s.car_engine_selected
        .iter()
        .zip(s.car_engine)
        .any(|(n, max)| *n != u32::MAX && *n > max)
        || s.wanted > 5
        || s.heist_cooldown
            .iter()
            .chain(s.robbery_cooldown.iter())
            .any(|v| !v.is_finite() || *v < 0. || *v > 900.)
        || s.car_engine.iter().any(|n| *n > 5)
        || s.car_brakes.iter().any(|n| *n > 3)
        || s.car_suspension.iter().any(|n| *n > 2)
        || s.car_paint
            .iter()
            .flatten()
            .any(|n| !n.is_finite() || *n < 0. || *n > 1.)
        || s.leases
            .iter()
            .any(|n| !n.is_finite() || *n < 0. || *n > 900.)
        || s.magazines.iter().zip([12, 6, 24]).any(|(v, max)| *v > max)
        || s.ammunition.iter().any(|v| *v > 1000)
        || s.texture_level > 3
        || s.run_level > 5
        || s.endurance_level > 5
        || s.carry_level > 5
        || s.water > 20
        || s.food > 20
        || s.quality > 3
        || !(50..=100).contains(&s.resolution_scale)
        || s.shader_effects > 4
        || s.shadow_quality > 3
        || s.owned.iter().any(|i| *i >= VEHICLE_SLOTS)
        || !s.position.x.is_finite()
        || !s.position.z.is_finite()
        || !s.position.y.is_finite()
        || !s.render_distance.is_finite()
        || s.render_distance < 80.
        || s.render_distance > 650.
        || s.ammo > 12
        || !s.sensitivity.is_finite()
        || s.sensitivity <= 0.
    {
        return Err("Invalid save".into());
    }
    Ok(s)
}
pub fn store(s: &Save) -> io::Result<()> {
    store_at(&path(), s)
}
fn store_at(p: &std::path::Path, s: &Save) -> io::Result<()> {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?
    }
    let tmp = p.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(s)?)?;
    replace_save(&tmp, p)
}

#[cfg(not(windows))]
fn replace_save(tmp: &std::path::Path, dest: &std::path::Path) -> io::Result<()> {
    std::fs::rename(tmp, dest)
}
#[cfg(windows)]
fn replace_save(tmp: &std::path::Path, dest: &std::path::Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(source: *const u16, dest: *const u16, flags: u32) -> i32;
    }
    fn wide(path: &std::path::Path) -> io::Result<Vec<u16>> {
        let mut value: Vec<u16> = path.as_os_str().encode_wide().collect();
        if value.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Save path contains a null character",
            ));
        }
        value.push(0);
        Ok(value)
    }
    let source = wide(tmp)?;
    let destination = wide(dest)?;
    // Replace existing saves without deleting the previous file first.
    if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 1 | 8) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn default_quality() -> usize {
    1
}
fn default_resolution() -> u8 {
    100
}
fn default_shadow_quality() -> u8 {
    1
}
fn default_shader_pack() -> String {
    "builtin".into()
}
fn default_shader_effects() -> u8 {
    4
}
fn default_ammo() -> u32 {
    0
}
fn default_reserve() -> u32 {
    0
}
fn default_distance() -> f32 {
    450.
}

fn default_shadows() -> bool {
    true
}

fn read_team<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Team, D::Error> {
    let team = Team::deserialize(d)?;
    Ok(if team == Team::Police {
        Team::Prisoner
    } else {
        team
    })
}

fn default_selected_engine() -> [u32; VEHICLE_SLOTS] {
    [u32::MAX; VEHICLE_SLOTS]
}

fn read_car_engine<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<[u32; VEHICLE_SLOTS], D::Error> {
    let values = Vec::<u32>::deserialize(d)?;
    if values.len() > VEHICLE_SLOTS {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [0; VEHICLE_SLOTS];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

fn read_car_engine_selected<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<[u32; VEHICLE_SLOTS], D::Error> {
    let values = Vec::<u32>::deserialize(d)?;
    if values.len() > VEHICLE_SLOTS {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [u32::MAX; VEHICLE_SLOTS];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

fn read_car_brakes<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<[u32; VEHICLE_SLOTS], D::Error> {
    let values = Vec::<u32>::deserialize(d)?;
    if values.len() > VEHICLE_SLOTS {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [0; VEHICLE_SLOTS];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

fn read_car_suspension<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<[u32; VEHICLE_SLOTS], D::Error> {
    let values = Vec::<u32>::deserialize(d)?;
    if values.len() > VEHICLE_SLOTS {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [0; VEHICLE_SLOTS];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

fn read_car_paint<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<[[f32; 3]; VEHICLE_SLOTS], D::Error> {
    let values = Vec::<[f32; 3]>::deserialize(d)?;
    if values.len() > VEHICLE_SLOTS {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [[0.; 3]; VEHICLE_SLOTS];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn six_vehicle_save_migrates_without_losing_upgrades() {
        let mut value = serde_json::to_value(super::Save::default()).unwrap();
        for field in [
            "car_engine",
            "car_engine_selected",
            "car_brakes",
            "car_suspension",
            "car_paint",
        ] {
            value[field].as_array_mut().unwrap().truncate(6);
        }
        value["car_engine"][0] = serde_json::json!(4);
        let migrated: super::Save = serde_json::from_value(value).unwrap();
        assert_eq!(migrated.car_engine[0], 4);
        assert_eq!(migrated.car_engine[6], 0);
        assert_eq!(migrated.car_engine_selected[6], u32::MAX);
    }

    #[test]
    fn seven_vehicle_save_preserves_every_existing_slot_and_defaults_akari() {
        let mut value = serde_json::to_value(super::Save::default()).unwrap();
        for field in [
            "car_engine",
            "car_engine_selected",
            "car_brakes",
            "car_suspension",
            "car_paint",
        ] {
            value[field].as_array_mut().unwrap().truncate(7);
        }
        value["car_engine"][6] = serde_json::json!(3);
        value["car_engine_selected"][6] = serde_json::json!(2);
        value["car_paint"][6] = serde_json::json!([0.2, 0.4, 0.6]);
        let mut migrated: super::Save = serde_json::from_value(value).unwrap();
        assert_eq!(migrated.car_engine[6], 3);
        assert_eq!(migrated.car_engine_selected[6], 2);
        assert_eq!(migrated.car_paint[6], [0.2, 0.4, 0.6]);
        assert_eq!(migrated.car_engine[7], 0);
        assert_eq!(migrated.car_engine_selected[7], u32::MAX);
        assert_eq!(migrated.car_paint[7], [0.; 3]);
        migrated.car_engine[7] = 4;
        migrated.car_paint[7] = [0.9, 0.1, 0.2];
        migrated.owned.push(7);
        let path = std::env::temp_dir().join(format!("jbrs-akari-{}.json", std::process::id()));
        super::store_at(&path, &migrated).unwrap();
        let loaded = super::load_from(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(loaded.car_engine[7], 4);
        assert_eq!(loaded.car_paint[7], [0.9, 0.1, 0.2]);
        assert_eq!(loaded.owned, vec![7]);
    }

    use super::*;
    #[test]
    fn weapons_leases_and_vehicle_plans_roundtrip() {
        let s = Save {
            weapon_owned: [false, true, true],
            magazines: [0, 5, 20],
            ammunition: [0, 10, 50],
            leases: [800., 0., 0., 900.],
            car_engine: [2, 0, 0, 0, 0, 0, 0, 0],
            ..Default::default()
        };
        let t: Save = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(t.magazines, [0, 5, 20]);
        assert_eq!(t.leases[0], 800.);
        assert_eq!(t.car_engine[0], 2);
    }

    #[test]
    fn legacy_police_player_migrates_to_prisoner_without_losing_progress() {
        let save = Save {
            team: Team::Police,
            balance: 731,
            owned: vec![2, 7],
            car_engine: [2, 0, 0, 0, 0, 0, 0, 4],
            run_level: 3,
            ..Default::default()
        };
        let mut json = serde_json::to_value(save).unwrap();
        json["team"] = serde_json::Value::String("Police".into());
        let loaded: Save = serde_json::from_value(json).unwrap();
        assert_eq!(loaded.team, Team::Prisoner);
        assert_eq!(loaded.balance, 731);
        assert_eq!(loaded.owned, vec![2, 7]);
        assert_eq!(loaded.car_engine[0], 2);
        assert_eq!(loaded.car_engine[7], 4);
        assert_eq!(loaded.run_level, 3);
    }
    #[test]
    fn settings_and_training_persist_and_legacy_defaults_are_compatible() {
        let save = Save {
            texture_level: 3,
            object_shadows: false,
            shader_pack: "Golden_Hour_v2".into(),
            auto_shader_pack: true,
            resolution_scale: 70,
            fps_counter: true,
            shader_effects: 3,
            shadow_quality: 2,
            run_level: 2,
            endurance_level: 3,
            carry_level: 4,
            water: 2,
            food: 1,
            ..Default::default()
        };
        let mut v = serde_json::to_value(&save).unwrap();
        let loaded: Save = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(loaded.texture_level, 3);
        assert!(!loaded.object_shadows);
        assert_eq!(loaded.shader_pack, "Golden_Hour_v2");
        assert!(loaded.auto_shader_pack);
        assert_eq!(loaded.resolution_scale, 70);
        assert!(loaded.fps_counter);
        assert_eq!(loaded.shader_effects, 3);
        assert_eq!(loaded.shadow_quality, 2);
        assert_eq!(loaded.carry_level, 4);
        assert_eq!(loaded.water, 2);
        for name in [
            "texture_level",
            "object_shadows",
            "shader_pack",
            "auto_shader_pack",
            "resolution_scale",
            "fps_counter",
            "shader_effects",
            "shadow_quality",
            "run_level",
            "endurance_level",
            "carry_level",
            "water",
            "food",
        ] {
            v.as_object_mut().unwrap().remove(name);
        }
        let legacy: Save = serde_json::from_value(v).unwrap();
        assert_eq!(legacy.texture_level, 0);
        assert!(legacy.object_shadows);
        assert_eq!(legacy.shader_pack, "builtin");
        assert!(!legacy.auto_shader_pack);
        assert_eq!(legacy.resolution_scale, 100);
        assert!(!legacy.fps_counter);
        assert_eq!(legacy.shader_effects, 4);
        assert_eq!(legacy.shadow_quality, 1);
        assert_eq!(legacy.run_level, 0);
    }
    #[test]
    fn purchased_weapon_and_quality_persist_legacy_has_no_free_gun() {
        let s = Save {
            pistol_owned: true,
            quality: 3,
            ammo: 8,
            reserve: 30,
            ..Default::default()
        };
        let mut json = serde_json::to_value(&s).unwrap();
        let loaded: Save = serde_json::from_value(json.clone()).unwrap();
        assert!(loaded.pistol_owned);
        assert_eq!(loaded.quality, 3);
        assert_eq!(loaded.ammo, 8);
        json.as_object_mut().unwrap().remove("pistol_owned");
        json.as_object_mut().unwrap().remove("quality");
        let legacy: Save = serde_json::from_value(json).unwrap();
        assert!(!legacy.pistol_owned);
        assert_eq!(legacy.quality, 1);
    }
    #[test]
    fn disk_roundtrip_and_version_rejection() {
        let filename = std::env::temp_dir().join(format!(
            "JBRS-test-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut s = Save {
            balance: 1234,
            ..Default::default()
        };
        store_at(&filename, &s).unwrap();
        assert_eq!(load_from(&filename).unwrap().balance, 1234);
        s.version = 99;
        store_at(&filename, &s).unwrap();
        assert!(load_from(&filename).is_err());
        std::fs::remove_file(filename).unwrap();
    }
    #[test]
    fn roundtrip() {
        let mut s = Save {
            balance: 2500,
            ..Default::default()
        };
        s.owned.push(3);
        s.team = Team::Criminal;
        s.color = [0.2, 0.4, 0.6];
        let bytes = serde_json::to_vec(&s).unwrap();
        let t: Save = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(t.balance, 2500);
        assert_eq!(t.owned, vec![3]);
        assert_eq!(t.team, Team::Criminal);
        assert_eq!(t.color, s.color);
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    fn directory(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("jbrs-recovery-{label}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
    #[test]
    fn invalid_settings_preserve_progress_and_exact_original_before_store() {
        let dir = directory("settings");
        let p = dir.join("save.json");
        let original = Save {
            balance: 91234,
            owned: vec![0, 7],
            quality: 99,
            render_distance: 1000.,
            car_engine: [3; VEHICLE_SLOTS],
            ..Default::default()
        };
        let bytes = serde_json::to_vec(&original).unwrap();
        std::fs::write(&p, &bytes).unwrap();
        let recovered = load_recovering(&p).unwrap();
        assert_eq!(recovered.balance, 91234);
        assert_eq!(recovered.owned, vec![0, 7]);
        assert_eq!(recovered.car_engine, [3; VEHICLE_SLOTS]);
        assert_eq!(recovered.quality, 3);
        assert_eq!(recovered.render_distance, 650.);
        assert_eq!(std::fs::read(&p).unwrap(), bytes);
        assert_eq!(
            std::fs::read(p.with_extension("recovery-1.json")).unwrap(),
            bytes
        );
        store_at(&p, &recovered).unwrap();
        assert_eq!(load_from(&p).unwrap().balance, 91234);
        assert_eq!(
            std::fs::read(p.with_extension("recovery-1.json")).unwrap(),
            bytes
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn malformed_save_starts_fresh_without_overwriting_any_backup() {
        let dir = directory("malformed");
        let p = dir.join("save.json");
        std::fs::write(&p, b"{broken").unwrap();
        std::fs::write(p.with_extension("recovery-1.json"), b"previous backup").unwrap();
        assert_eq!(load_recovering(&p).unwrap().balance, 0);
        assert_eq!(
            std::fs::read(p.with_extension("recovery-1.json")).unwrap(),
            b"previous backup"
        );
        assert_eq!(
            std::fs::read(p.with_extension("recovery-2.json")).unwrap(),
            b"{broken"
        );
        assert_eq!(std::fs::read(&p).unwrap(), b"{broken");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn valid_save_is_loaded_without_backup_and_io_errors_are_not_reset() {
        let dir = directory("valid");
        let p = dir.join("save.json");
        let save = Save {
            balance: 533,
            ..Default::default()
        };
        store_at(&p, &save).unwrap();
        assert_eq!(load_recovering(&p).unwrap().balance, 533);
        assert!(!p.with_extension("recovery-1.json").exists());
        assert!(load_recovering(&dir).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
