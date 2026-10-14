use engine_core::{Team, Vec3};
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
    pub car_engine: [u32; 7],
    #[serde(default = "default_selected_engine")]
    #[serde(deserialize_with = "read_car_engine_selected")]
    pub car_engine_selected: [u32; 7],
    #[serde(default)]
    #[serde(deserialize_with = "read_car_brakes")]
    pub car_brakes: [u32; 7],
    #[serde(default)]
    #[serde(deserialize_with = "read_car_suspension")]
    pub car_suspension: [u32; 7],
    #[serde(default)]
    #[serde(deserialize_with = "read_car_paint")]
    pub car_paint: [[f32; 3]; 7],
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
    pub version: u32,
    pub balance: u32,
    pub owned: Vec<usize>,
    pub color: [f32; 3],
    pub position: Vec3,
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
            car_engine: [0; 7],
            car_engine_selected: [u32::MAX; 7],
            car_brakes: [0; 7],
            car_suspension: [0; 7],
            car_paint: [[0.; 3]; 7],
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
    if let Some(p) = std::env::var_os("JAILBREAK_SAVE") {
        return p.into();
    }
    let mut p = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
        });
    p.push("jailbreak-rs/save.json");
    p
}
pub fn load() -> Result<Save, String> {
    load_from(&path())
}
fn load_from(p: &std::path::Path) -> Result<Save, String> {
    if !p.exists() {
        return Ok(Save::default());
    }
    let s: Save = serde_json::from_slice(&std::fs::read(p).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
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
        || s.owned.iter().any(|i| *i >= 6)
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
    std::fs::rename(tmp, p)
}

fn default_quality() -> usize {
    1
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

fn default_selected_engine() -> [u32; 7] {
    [u32::MAX; 7]
}

fn read_car_engine<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[u32; 7], D::Error> {
    let values = Vec::<u32>::deserialize(d)?;
    if values.len() > 7 {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [0; 7];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

fn read_car_engine_selected<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[u32; 7], D::Error> {
    let values = Vec::<u32>::deserialize(d)?;
    if values.len() > 7 {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [u32::MAX; 7];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

fn read_car_brakes<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[u32; 7], D::Error> {
    let values = Vec::<u32>::deserialize(d)?;
    if values.len() > 7 {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [0; 7];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

fn read_car_suspension<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[u32; 7], D::Error> {
    let values = Vec::<u32>::deserialize(d)?;
    if values.len() > 7 {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [0; 7];
    for (i, v) in values.into_iter().enumerate() {
        out[i] = v;
    }
    Ok(out)
}

fn read_car_paint<'de, D: serde::Deserializer<'de>>(d: D) -> Result<[[f32; 3]; 7], D::Error> {
    let values = Vec::<[f32; 3]>::deserialize(d)?;
    if values.len() > 7 {
        return Err(serde::de::Error::custom("too many vehicle slots"));
    }
    let mut out = [[0.; 3]; 7];
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

    use super::*;
    #[test]
    fn weapons_leases_and_vehicle_plans_roundtrip() {
        let s = Save {
            weapon_owned: [false, true, true],
            magazines: [0, 5, 20],
            ammunition: [0, 10, 50],
            leases: [800., 0., 0., 900.],
            car_engine: [2, 0, 0, 0, 0, 0, 0],
            ..Default::default()
        };
        let t: Save = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(t.magazines, [0, 5, 20]);
        assert_eq!(t.leases[0], 800.);
        assert_eq!(t.car_engine[0], 2);
    }
    #[test]
    fn settings_and_training_persist_and_legacy_defaults_are_compatible() {
        let save = Save {
            texture_level: 3,
            object_shadows: false,
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
        assert_eq!(loaded.carry_level, 4);
        assert_eq!(loaded.water, 2);
        for name in [
            "texture_level",
            "object_shadows",
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
            "jailbreak-test-{}-{}.json",
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
