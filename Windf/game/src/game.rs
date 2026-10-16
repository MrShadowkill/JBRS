use crate::npcs::{Npc, Role};
use engine_core::{Team, Vec3, STEP};
use gui::text;
use player::Player;
use renderer_gl21::Renderer;
use robberies::alarm::{Alarm, Outcome, SITES};
use vehicles::{Car, CATALOG};
use world::World;
const WHITE: [f32; 3] = [0.9, 0.94, 1.];
const GOLD: [f32; 3] = [1., 0.78, 0.24];
struct Game {
    world: std::sync::Arc<World>,
    view: Option<crate::world_view::WorldView>,
    captured: bool,
    capture_frame: usize,
    scene_triangles: usize,
    vehicle_triangles: usize,
    npc_triangles: usize,
    capture_at: f32,
    player: Player,
    cars: Vec<Car>,
    car_models: &'static [[vehicles::body::Model; 2]],
    driving: Option<usize>,
    save: save_system::Save,
    alarm: Alarm,
    npcs: Vec<Npc>,
    doors: Vec<crate::doors::Door>,
    slot: usize,
    explosive: bool,
    charge_taken: bool,
    fuse: Option<f32>,
    breached: bool,
    stun: f32,
    blast: f32,
    response: Option<(Vec<Vec3>, usize)>,
    wanted: u32,
    cuffs: crate::escape::Cuffs,
    immunity: f32,
    bounty: u32,
    random: u64,
    drops: Vec<(Vec3, bool)>,
    parachute: bool,
    heists: crate::heists::Heists,
    yaw: f32,
    pitch: f32,
    menu: bool,
    map: bool,
    garage: bool,
    workshop: Option<usize>,
    workshop_config: bool,
    training: bool,
    graphics_row: usize,
    texture_applied: usize,
    stamina: f32,
    holding: bool,
    punch_time: f32,
    punch_done: bool,
    movement: f32,
    camera: Option<[f32; 3]>,
    training_time: f32,
    armory: bool,
    weapon_selection: usize,
    housing: Option<usize>,
    cursor: bool,
    stance: u8,
    prone: bool,
    crouched: bool,

    message: String,
    message_time: f32,
    clock: f32,
    stats: profiling::Stats,
    shot: f32,
    debug: bool,
    waypoint: Option<Vec3>,
    pistol: weapons::Pistol,
}
impl Game {
    fn new() -> Result<Self, String> {
        Self::from_save(save_system::load()?)
    }
    fn from_save(mut save: save_system::Save) -> Result<Self, String> {
        for i in 0..CATALOG.len() {
            if save.car_engine_selected[i] == u32::MAX {
                save.car_engine_selected[i] = save.car_engine[i];
            }
        }
        save.weapon_owned[0] |= save.pistol_owned;
        save.magazines[0] = save.ammo;
        save.ammunition[0] = save.reserve;
        let mut player = Player::new(save.team);
        player.pos = save.position;
        let pistol = weapons::Pistol {
            ammo: if save.pistol_owned { save.ammo } else { 0 },
            reserve: if save.pistol_owned { save.reserve } else { 0 },
            ..Default::default()
        };
        let world = std::sync::Arc::new(World::new());
        if !world::prison::underground(player.pos) {
            player.pos.y = world.walking_surface(player.pos) + 1.;
        }
        let doors = world
            .buildings
            .iter()
            .map(|_| crate::doors::Door::new())
            .collect();
        let mut cars = vec![
            Car::new(0, Vec3::new(-95., 0.65, -76.)),
            Car::new(1, Vec3::new(-74., 0.65, -76.)),
            Car::new(2, Vec3::new(0., 0.65, -140.)),
            Car::new(3, Vec3::new(750., 0.65, -430.)),
            Car::new(4, Vec3::new(560., 0.65, 175.)),
            Car::new(5, Vec3::new(430., 0.65, -72.)),
            Car::new(6, Vec3::new(420., 0.65, 330.)),
            Car::new(7, Vec3::new(440., 0.65, -72.)),
        ];
        for car in &mut cars {
            car.pos.y = world.surface(car.pos.x, car.pos.z) + 0.65;
            configure_car(car, &save);
        }
        let mut heists = crate::heists::Heists::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
        heists.cooldown = save.heist_cooldown;
        let wanted = save.wanted;
        let bounty = save.bounty;
        let alarm = Alarm {
            cooldown: save.robbery_cooldown,
            ..Default::default()
        };
        Ok(Self {
            world: world.clone(),
            doors,
            slot: 0,
            explosive: false,
            charge_taken: false,
            fuse: None,
            breached: false,
            stun: 0.,
            blast: 0.,
            response: None,
            view: None,
            captured: false,
            capture_frame: 0,
            scene_triangles: 0,
            vehicle_triangles: 0,
            npc_triangles: 0,
            capture_at: 3.,
            player,
            cars,
            car_models: vehicles::body::catalog()?,
            driving: None,
            save,
            alarm,
            npcs: {
                let mut npcs = crate::npcs::population();
                for n in &mut npcs {
                    n.pos.y = world.walking_surface(n.pos) + 1.;
                    n.home.y = n.pos.y;
                }
                npcs
            },
            wanted,
            cuffs: Default::default(),
            immunity: 0.,
            bounty,
            random: 0x6a09e667
                ^ std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            drops: Vec::new(),
            parachute: false,
            heists,
            yaw: std::f32::consts::PI,
            pitch: 0.32,
            menu: true,
            map: false,
            garage: false,
            workshop: None,
            workshop_config: false,
            training: false,
            graphics_row: 0,
            texture_applied: usize::MAX,
            stamina: 100.,
            holding: false,
            punch_time: 0.,
            punch_done: false,
            movement: 0.,
            camera: None,
            training_time: 0.,
            armory: false,
            weapon_selection: 0,
            housing: None,
            cursor: false,
            stance: 0,
            prone: false,
            crouched: false,

            message: "PRISON LOCKED - SEWER MAZE OR C4; P PARACHUTE".into(),
            message_time: 12.,
            clock: 0.,
            stats: profiling::Stats::new(),
            shot: 0.,
            debug: false,
            waypoint: None,
            pistol,
        })
    }
    fn notify(&mut self, s: &str) {
        self.message = s.into();
        self.message_time = 8.;
    }
    fn save(&mut self) {
        self.save.heist_cooldown = self.heists.cooldown;
        self.save.robbery_cooldown = self.alarm.cooldown;
        // Save-and-exit cannot bypass a robbery lockout; unbanked loot is forfeited.
        if let Some(id) = self.heists.site {
            self.save.heist_cooldown[id] = 900.;
        }
        if let Some(id) = self.alarm.site {
            self.save.robbery_cooldown[id] = 900.;
        }
        self.save.wanted = self.wanted.min(5);
        self.save.bounty = self.bounty;
        self.save.magazines[self.pistol.kind] = self.pistol.ammo;
        self.save.ammunition[self.pistol.kind] = self.pistol.reserve;
        self.save.ammo = self.save.magazines[0];
        self.save.reserve = self.save.ammunition[0];
        self.save.position = self.player.pos;
        self.save.team = self.player.team;
        match save_system::store(&self.save) {
            Ok(()) => self.notify("PROGRESS SAVED"),
            Err(e) => self.notify(&format!("SAVE FAILED: {e}")),
        }
    }
    fn blocked(&self, p: Vec3, r: f32) -> bool {
        self.world.blocked(p, r)
            || world::prison::barrier(p, r, self.breached)
            || self
                .doors
                .iter()
                .zip(&self.world.buildings)
                .any(|(door, b)| door.blocked(b, p, r))
    }
    fn clear(&self, a: Vec3, b: Vec3) -> bool {
        if (a.y - b.y).abs() > 2. {
            return false;
        }
        let n = (a.distance(b) / 0.35).ceil().max(1.) as usize;
        (1..n).all(|j| {
            let t = j as f32 / n as f32;
            !self.blocked(
                Vec3::new(a.x + (b.x - a.x) * t, a.y, a.z + (b.z - a.z) * t),
                0.1,
            )
        })
    }
    fn key_action(&mut self, k: i32) -> bool {
        if self.heists.quiz && !self.cuffs.active {
            match k {
                48..=57 => {
                    if self.heists.answer.len() < 8 {
                        self.heists.answer.push(char::from_u32(k as u32).unwrap());
                    }
                }
                8 => {
                    self.heists.answer.pop();
                }
                13 if !self.heists.submit() => self.notify("WRONG ANSWER - TRY AGAIN"),
                _ => {}
            }
            return true;
        }
        if self.cuffs.active {
            if k == 32 && self.cuffs.hit() {
                self.immunity = 12.;
                self.stun = 0.;
                self.notify("ESCAPED CUFFS - 12S ARREST / STUN IMMUNITY");
            }
            return true;
        }
        match k {
            112 if !self.modal() => self.parachute = !self.parachute,
            9 => self.cursor = !self.cursor,
            122 if !self.modal() => {
                self.prone = !self.prone;
                self.crouched = false;
            }
            99 if !self.modal()
                && !self.world.interior(self.player.pos).is_some_and(|id| {
                    matches!(
                        self.world.buildings[id].name,
                        "Grocery" | "Donut Shop" | "Gas Station"
                    )
                }) =>
            {
                self.crouched = !self.crouched;
                self.prone = false;
            }
            49..=51 if self.armory => self.weapon_selection = (k - 49) as usize,
            13 | 98 if self.armory => {
                self.buy_weapon(self.weapon_selection);
            }
            49 if self.housing.is_some() => self.lease(self.housing.unwrap()),
            50 if self.housing.is_some() => {
                let id = self.housing.unwrap();
                self.housing = None;
                self.slot = 0;
                if id >= 2 {
                    self.notify("RENT THIS HOME TO REST HERE");
                    return true;
                }
                self.notify("FACE THE CASE - F TO BREAK THE LOCK");
            }
            51 if self.housing.is_some() => {
                if self.save.leases[self.housing.unwrap()] > 0. {
                    self.player.health = 100.;
                    self.stamina = 100. + self.save.endurance_level as f32 * 20.;
                    self.notify("RESTED AT HOME");
                }
            }
            27 if self.modal() => self.close_panels(),
            27 => {
                self.menu = !self.menu;
                self.workshop = None;
                self.training = false;
                self.armory = false;
                self.housing = None;
            }
            13 if self.workshop.is_some() && self.workshop_config => self.save(),
            13 => {
                self.menu = false;
                self.garage = false;
                self.armory = false;
                self.housing = None;
                self.workshop = None;
                self.training = false;
                self.armory = false;
                self.housing = None;
            }
            110 if self.workshop.is_some() => self.workshop_config = !self.workshop_config,
            1073741903 if self.workshop.is_some() && self.workshop_config => {
                self.workshop = Some((self.workshop.unwrap() + 1) % CATALOG.len())
            }
            1073741904 if self.workshop.is_some() && self.workshop_config => {
                self.workshop = Some((self.workshop.unwrap() + CATALOG.len() - 1) % CATALOG.len())
            }

            48..=53 if self.workshop.is_some() && self.workshop_config => {
                self.set_engine((k - 48) as u32)
            }
            49..=53 if self.workshop.is_some() => self.service((k - 48) as u32),
            49..=51 if self.training => self.train((k - 48) as u32),
            49..=51 if self.menu => {
                self.player = Player::new(match k {
                    49 => Team::Prisoner,
                    50 => Team::Police,
                    _ => Team::Criminal,
                });
                self.driving = None;
                self.wanted = 0;
                self.bounty = 0;
                self.heists.bag = 0;
                self.heists.finish();
                self.cuffs.active = false;
                self.immunity = 0.;
                self.camera = None;
                self.alarm.cancel();
                self.menu = false;
            }
            101 if !self.modal() => self.interact(),
            109 => {
                let open = !self.map;
                self.close_panels();
                self.map = open;
            }
            106 => {
                if self.workshop.is_some() {
                    self.close_panels();
                } else {
                    self.close_panels();
                    self.open_workshop();
                }
            }
            103 => {
                let open = !self.garage;
                self.close_panels();
                self.garage = open;
            }
            98 if !self.menu
                && !self.garage
                && !self.map
                && self.world.interior(self.player.pos).is_some_and(|id| {
                    matches!(
                        self.world.buildings[id].name,
                        "Grocery" | "Donut Shop" | "Gas Station"
                    )
                }) =>
            {
                self.supplies(true)
            }
            99 if !self.menu && !self.garage && !self.map => self.supplies(false),
            98 if !self.menu && self.save.pistol_owned => {
                if self
                    .world
                    .interior(self.player.pos)
                    .is_some_and(|id| self.world.buildings[id].name == "Weapons")
                    && self.save.balance >= 50
                {
                    self.save.balance -= 50;
                    self.pistol.reserve =
                        (self.pistol.reserve + 36).min(36 + self.save.carry_level * 24);
                    self.notify("36 ROUNDS PURCHASED - $50");
                } else {
                    self.notify("B AT WEAPONS: 36 ROUNDS COST $50");
                }
            }

            1073741906 if self.garage => self.graphics_row = (self.graphics_row + 2) % 3,
            1073741905 if self.garage => self.graphics_row = (self.graphics_row + 1) % 3,
            1073741904 | 1073741903 if self.garage => {
                let d = if k == 1073741904 { 3 } else { 1 };
                match self.graphics_row {
                    0 => self.save.quality = (self.save.quality + d) % 4,
                    1 => self.save.texture_level = (self.save.texture_level + d) % 4,
                    _ => self.save.object_shadows = !self.save.object_shadows,
                }
            }
            1073741886 => self.save(),
            1073741884 => self.debug = !self.debug,
            113 if self.menu => return false,
            49..=55 if !self.modal() => {
                self.slot = (k - 49) as usize;
                match self.slot {
                    1 => self.equip_weapon(0),
                    5 => self.equip_weapon(1),
                    6 => self.equip_weapon(2),
                    _ => {}
                }
            }
            108 => self.save.quality = (self.save.quality + 1) % 4,
            114 if !self.modal() && matches!(self.slot, 1 | 5 | 6) => self.pistol.reload(),
            102 if !self.modal() => self.fire(),
            _ => {}
        }
        true
    }
    fn modal(&self) -> bool {
        self.menu
            || self.map
            || self.garage
            || self.workshop.is_some()
            || self.training
            || self.armory
            || self.housing.is_some()
    }
    fn close_panels(&mut self) {
        self.menu = false;
        self.map = false;
        self.garage = false;
        self.workshop = None;
        self.training = false;
        self.armory = false;
        self.housing = None;
    }
    fn equip_weapon(&mut self, id: usize) {
        self.save.magazines[self.pistol.kind] = self.pistol.ammo;
        self.save.ammunition[self.pistol.kind] = self.pistol.reserve;
        self.pistol = weapons::Pistol {
            kind: id,
            ammo: self.save.magazines[id],
            reserve: self.save.ammunition[id],
            ..Default::default()
        };
        self.slot = match id {
            0 => 1,
            1 => 5,
            _ => 6,
        };
    }
    fn buy_weapon(&mut self, id: usize) -> bool {
        let spec = weapons::CATALOG[id];
        if self.save.weapon_owned[id] {
            self.equip_weapon(id);
            self.notify("ALREADY OWNED - EQUIPPED");
            return false;
        }
        if self.save.balance < spec.price {
            self.notify("NOT ENOUGH CASH FOR THIS WEAPON");
            return false;
        }
        self.save.balance -= spec.price;
        self.save.weapon_owned[id] = true;
        if id == 0 {
            self.save.pistol_owned = true;
        }
        self.save.magazines[self.pistol.kind] = self.pistol.ammo;
        self.save.ammunition[self.pistol.kind] = self.pistol.reserve;
        self.pistol = weapons::Pistol {
            kind: id,
            ammo: spec.magazine,
            reserve: spec.reserve,
            ..Default::default()
        };
        self.save.magazines[id] = spec.magazine;
        self.save.ammunition[id] = spec.reserve;
        self.slot = match id {
            0 => 1,
            1 => 5,
            _ => 6,
        };
        self.notify(&format!("{} PURCHASED ${}", spec.name, spec.price));
        true
    }
    fn buy_pistol(&mut self) -> bool {
        self.buy_weapon(0)
    }
    fn lease(&mut self, id: usize) {
        let price = [150, 180, 500, 100][id];
        if self.save.balance < price {
            self.notify("NOT ENOUGH CASH TO RENT");
            return;
        }
        self.save.balance -= price;
        self.save.leases[id] = 900.;
        self.notify("RENTED FOR 15 MINUTES OF PLAY - 3 REST AT HOME");
    }
    fn property(&self) -> Option<usize> {
        let p = self.player.pos;
        [(-220., 210.), (-300., 260.), (350., -50.), (-335., 320.)]
            .iter()
            .position(|&(x, z)| p.distance(Vec3::new(x, 0., z)) < 14.)
    }
    fn player_blocked(&self, p: Vec3, r: f32) -> bool {
        if self.stance == 0 {
            self.blocked(p, r)
        } else {
            self.world
                .body_blocked(p, r, if self.stance == 2 { 0.65 } else { 1.6 })
                || world::prison::barrier(p, r, self.breached)
                || self
                    .doors
                    .iter()
                    .zip(&self.world.buildings)
                    .any(|(d, b)| d.blocked(b, p, r))
        }
    }
    fn steal(&mut self, id: usize) {
        if self.response.as_ref().is_some_and(|(_, car)| *car == id) {
            self.response = None;
        }
        self.camera = None;
        self.driving = Some(id);
        self.player.team = Team::Criminal;
        self.wanted = (self.wanted + 1).min(5);
        self.bounty += 100;
        self.yaw = std::f32::consts::PI - self.cars[id].angle;
        self.notify("VEHICLE STOLEN - YOU ARE NOW A CRIMINAL - E EXIT");
    }
    fn arrest(&mut self) {
        self.cuffs.active = false;
        self.immunity = 0.;
        self.bounty = 0;
        self.heists.bag = 0;
        self.heists.finish();
        self.camera = None;
        self.player = Player::new(Team::Prisoner);
        self.driving = None;
        self.wanted = 0;
        self.alarm.cancel();
        self.save.pistol_owned = false;
        self.save.weapon_owned = [false; 3];
        self.save.magazines = [0; 3];
        self.save.ammunition = [0; 3];
        self.close_panels();
        self.pistol.ammo = 0;
        self.pistol.reserve = 0;
        self.slot = 0;
        self.explosive = false;
        self.charge_taken = false;
        self.fuse = None;
        self.breached = false;
        self.stun = 0.;
        self.response = None;
        self.workshop = None;
        self.training = false;
        self.punch_time = 0.;
        self.notify("ARRESTED - LOOT AND CONTRABAND CONFISCATED - BACK TO PRISON");
    }
    fn interact(&mut self) {
        if self.stun > 0. || self.cuffs.active {
            return;
        }
        if let Some(i) = self
            .drops
            .iter()
            .position(|(q, taken)| !taken && q.distance(self.player.pos) < 3.)
        {
            self.drops[i].1 = true;
            self.explosive = true;
            self.notify("C4 FOUND ON BODY");
            return;
        }
        if let Some(i) = self.driving {
            let c = &self.cars[i];
            for side in [1., -1.] {
                let mut p = Vec3::new(
                    c.pos.x + side * c.angle.cos() * 3.,
                    c.pos.y,
                    c.pos.z - side * c.angle.sin() * 3.,
                );
                p.y = self.world.surface(p.x, p.z) + 1.;
                if !self.blocked(p, 0.5) {
                    self.player.pos = p;
                    self.player.vy = 0.;
                    self.driving = None;
                    self.camera = None;
                    break;
                }
            }
            return;
        }
        let p = self.player.pos;
        if self.special_interact() {
            return;
        }
        if p.distance(Vec3::new(350., 1., -65.)) < 3. && p.y < 8. {
            self.player.pos = Vec3::new(350., 19., -58.);
            self.notify("PENTHOUSE LIFT - E HERE TO GO DOWN");
            return;
        }
        if p.distance(Vec3::new(350., 19., -58.)) < 3. && p.y > 15. {
            self.player.pos = Vec3::new(350., 1., -66.);
            self.notify("LIFT TO STREET");
            return;
        }
        for hatch in [world::prison::HATCH, world::prison::EXIT] {
            if p.distance(hatch) < 2.5 {
                self.player.pos = Vec3::new(hatch.x, if p.y < -0.5 { 1. } else { -3. }, hatch.z);
                self.player.vy = 0.;
                self.notify("SEWER LADDER - WALK TO THE OTHER HATCH - E CLIMB");
                return;
            }
        }
        if !self.charge_taken && p.distance(world::prison::CHARGE) < 3. && p.y > 0. {
            self.explosive = true;
            self.charge_taken = true;
            self.slot = 2;
            self.notify("C4 FOUND - SLOT 3 - E AT CRACKED WEST WALL");
            return;
        }
        if self.slot == 2 && self.explosive && p.distance(world::prison::WEAK) < 4. {
            self.explosive = false;
            self.fuse = Some(3.);
            self.notify("C4 PLANTED - 3 SECONDS - STEP BACK");
            return;
        }
        if let Some(id) = self.world.buildings.iter().position(|b| {
            b.interior && p.distance(Vec3::new(b.p.x, 1., b.p.z - b.size.z / 2.)) < 4.5 && p.y > 0.
        }) {
            self.doors[id].open = !self.doors[id].open;
            self.notify("DOOR OPEN / CLOSE");
            return;
        }
        if let Some(id) = self.world.interior(p) {
            let name = self.world.buildings[id].name;
            if name == "Weapons" || name.contains("Police") {
                self.armory = true;
                self.weapon_selection = 0;
                return;
            }
            if matches!(name, "Garage" | "Desert Garage") {
                self.open_workshop();
                return;
            }
            if name == "Gym" {
                self.training = true;
                return;
            }
            if let Some(id) = self.property() {
                if self.alarm.site.is_some() && self.alarm.hits.iter().any(|n| *n >= 3) {
                    self.notify("HOLD E TO COLLECT");
                    return;
                }
                self.housing = Some(id);
                return;
            }
            if matches!(name, "Criminal Base" | "Safehouse") {
                self.notify("SAFEHOUSE - GYM NEARBY - NO FREE GUNS");
                return;
            }
            if matches!(name, "Canteen" | "Infirmary") {
                self.player.health = 100.;
                self.notify("HEALTH RESTORED");
                return;
            }
            if let Some(site) = SITES
                .iter()
                .position(|site| site.center.distance(self.world.buildings[id].p) < 1.)
            {
                if self.player.team == Team::Police {
                    self.notify("POLICE CANNOT ROB STORES");
                    return;
                }
                self.notify(if self.alarm.hits[site] < 3 {
                    "SLOT 1 - F OR LEFT CLICK: HIT THE GLASS 3 TIMES"
                } else {
                    "HOLD E NEAR THE BROKEN CASE TO TAKE LOOT"
                });
                return;
            }
        }
        if self.player.team == Team::Police {
            if let Some(id) = self.npcs.iter().position(|n| {
                n.role == Role::Robber
                    && n.disabled <= 0.
                    && n.pos.distance(p) < 3.
                    && self.clear(p, n.pos)
            }) {
                self.npcs[id].disabled = 60.;
                self.save.balance += 50;
                self.notify("ROBBER ARRESTED - $50");
                return;
            }
        }
        if let Some(id) = self
            .cars
            .iter()
            .position(|c| p.distance(c.pos) < 4.5 && (p.y - c.pos.y).abs() < 2.)
        {
            self.steal(id);
        }
    }
    fn prepare_response(&mut self, site: usize) {
        let target = SITES[site].center;
        let road = self
            .world
            .roads
            .iter()
            .min_by(|a, b| a.distance(target).0.total_cmp(&b.distance(target).0))
            .unwrap();
        let end = road
            .points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.distance(target).total_cmp(&b.distance(target)))
            .unwrap()
            .0;
        let (start, end) = if end >= 12 {
            (end - 12, end)
        } else {
            (end, (end + 12).min(road.points.len() - 1))
        };
        let mut route = road.points[start..=end].to_vec();
        if route.last().unwrap().distance(target) > route[0].distance(target) {
            route.reverse();
        }
        let b = self
            .world
            .buildings
            .iter()
            .min_by(|a, b| a.p.distance(target).total_cmp(&b.p.distance(target)))
            .unwrap();
        let mut curb = b.entrance();
        curb.z -= 4.;
        curb.y = self.world.surface(curb.x, curb.z);
        route.push(curb);
        let mut car = Car::new(2, route[0]);
        configure_car(&mut car, &self.save);
        car.pos.y += 0.65;
        let id = self.cars.len();
        self.cars.push(car);
        self.response = Some((route, id));
    }
    fn punch(&mut self) {
        if self.punch_time > 0.
            || self.stun > 0.
            || self.driving.is_some()
            || self.training_time > 0.
        {
            return;
        }
        self.player.heading = std::f32::consts::PI - self.yaw;
        self.punch_time = 0.55;
        self.punch_done = false;
    }
    fn punch_impact(&mut self) {
        if self.special_strike() {
            return;
        }
        let p = self.player.pos;
        for id in 0..SITES.len() {
            let case = Alarm::case(id);
            if id >= 5 && self.save.leases[id - 5] > 0. && p.distance(case) < 2.6 {
                self.notify("YOUR RENTED HOME CANNOT BE ROBBED");
                continue;
            }
            let a = self.player.heading;
            let dx = case.x - p.x;
            let dz = case.z - p.z;
            let facing = dx * a.sin() + dz * a.cos();
            if p.distance(case) < 2.6
                && facing > 0.3
                && self.player.team != Team::Police
                && self.clear(p, case)
            {
                let starting = self.alarm.site.is_none();
                if self.alarm.strike(id) {
                    self.player.team = Team::Criminal;
                    self.wanted = self.wanted.max(3);
                    if starting {
                        self.prepare_response(id);
                    }
                    self.notify(if self.alarm.hits[id] >= 3 {
                        "GLASS BROKEN - HOLD E TO TAKE LOOT - ESCAPE BEFORE 30S"
                    } else {
                        "GLASS CRACKED - ALARM 30S - HIT AGAIN"
                    });
                }
                return;
            }
        }
        if let Some(id) = self
            .npcs
            .iter()
            .position(|n| n.disabled <= 0. && n.pos.distance(p) < 2. && self.clear(p, n.pos))
        {
            self.npcs[id].health -= 15.;
            self.drop_c4(id);
            self.player.team = Team::Criminal;
            self.wanted = self.wanted.max(1);
            self.notify("ASSAULT - POLICE ALERTED");
        }
    }
    fn open_workshop(&mut self) {
        let near = self.world.buildings.iter().any(|b| {
            matches!(b.name, "Garage" | "Desert Garage") && self.player.pos.distance(b.p) < 27.
        });
        if !near {
            self.notify("J SERVICE: BRING A STOLEN CAR TO THE GARAGE");
            return;
        }
        let id = self.driving.or_else(|| {
            self.cars
                .iter()
                .position(|c| c.pos.distance(self.player.pos) < 6.)
        });
        if let Some(id) = id {
            self.workshop = Some(id);
        } else {
            self.notify("PARK YOUR STOLEN CAR NEAR THE GARAGE");
        }
    }
    fn service(&mut self, option: u32) {
        let Some(id) = self.workshop else {
            return;
        };
        if option == 3 {
            let target = self.cars[id].engine_level + 1;
            if target > 5 {
                self.notify("UPGRADE ALREADY AT MAXIMUM");
                return;
            }
            self.set_engine(target);
            return;
        }
        let car = &self.cars[id];
        let (level, cap) = match option {
            3 => (car.engine_level, 5),
            4 => (car.brake_level, 3),
            5 => (car.suspension_level, 2),
            _ => (0, 1),
        };
        if level >= cap {
            self.notify("UPGRADE ALREADY AT MAXIMUM");
            return;
        }
        let cost = match option {
            1 => 40,
            2 => 25,
            3 => 150 + level * 100,
            4 => 100 + level * 80,
            _ => 120 + level * 100,
        };
        if self.save.balance < cost {
            self.notify("NOT ENOUGH CASH");
            return;
        }
        self.save.balance -= cost;
        if option == 1 {
            self.cars[id].damage = 0.;
        } else if option == 2 {
            let c = &mut self.cars[id];
            c.color = if c.color[0] > 0.5 {
                [0.17, 0.3, 0.42]
            } else {
                [0.68, 0.19, 0.12]
            };
        }
        if option == 3 {
            self.cars[id].engine_level += 1;
        } else if option == 4 {
            self.cars[id].brake_level += 1;
        } else if option == 5 {
            self.cars[id].suspension_level += 1;
        }
        let c = &self.cars[id];
        self.save.car_engine[c.kind] = self.save.car_engine[c.kind].max(c.engine_level);
        self.save.car_engine_selected[c.kind] = c.engine_level;
        self.save.car_brakes[c.kind] = c.brake_level;
        self.save.car_suspension[c.kind] = c.suspension_level;
        self.save.car_paint[c.kind] = c.color;
        self.notify("SERVICE COMPLETE");
    }
    fn train(&mut self, kind: u32) {
        let level = match kind {
            1 => self.save.run_level,
            2 => self.save.endurance_level,
            _ => self.save.carry_level,
        };
        let cost = 60 + level * 40;
        if self.training_time > 0. || level >= 5 || self.save.balance < cost {
            self.notify("TRAINING: MAX LEVEL OR NOT ENOUGH CASH");
            return;
        }
        self.save.balance -= cost;
        match kind {
            1 => self.save.run_level += 1,
            2 => self.save.endurance_level += 1,
            _ => self.save.carry_level += 1,
        };
        self.training = false;
        self.training_time = 3.;
        self.notify("TRAINING 3S - SKILL UPGRADED");
    }
    fn supplies(&mut self, water: bool) {
        let store = self.world.interior(self.player.pos).is_some_and(|id| {
            matches!(
                self.world.buildings[id].name,
                "Grocery" | "Donut Shop" | "Gas Station"
            )
        });
        let cost = if water { 8 } else { 12 };
        let stock = if water {
            self.save.water
        } else {
            self.save.food
        };
        let vendor = self.npcs.iter().any(|n| {
            n.role == Role::Vendor
                && n.pos.distance(self.player.pos) < 5.
                && self.clear(self.player.pos, n.pos)
        });
        if !store
            || !vendor
            || self.save.balance < cost
            || stock >= 20
            || self.save.water + self.save.food >= 8 + self.save.carry_level * 2
        {
            self.notify("AT STORE: B WATER $8 / C FOOD $12 - MAX 20");
            return;
        }
        self.save.balance -= cost;
        if water {
            self.save.water += 1;
        } else {
            self.save.food += 1;
        }
        self.notify("SUPPLIES PURCHASED - SLOT 4/5 - F USE");
    }
    fn fire(&mut self) {
        if self.heists.quiz || self.cuffs.active {
            return;
        }

        if self.slot == 0 {
            self.punch();
            return;
        }
        if self.slot == 3 && self.save.water > 0 {
            self.save.water -= 1;
            self.stamina = 100. + self.save.endurance_level as f32 * 20.;
            self.notify("WATER - STAMINA RESTORED");
            return;
        }
        if self.slot == 4 && self.save.food > 0 {
            self.save.food -= 1;
            self.player.health = (self.player.health + 25.).min(100.);
            self.notify("FOOD +25 HP");
            return;
        }

        if self.stun > 0.
            || !matches!(self.slot, 1 | 5 | 6)
            || !self.save.weapon_owned[self.pistol.kind]
        {
            self.notify("NO PISTOL EQUIPPED - BUY FOR $350 AT WEAPONS");
            return;
        }
        if !self.pistol.fire() {
            self.notify("R RELOAD");
            return;
        }
        self.shot = 0.15;
        if self.player.team != Team::Police {
            self.player.team = Team::Criminal;
            self.wanted = self.wanted.max(2);
        }
        let p = self.player.pos;
        let victim = self
            .npcs
            .iter()
            .enumerate()
            .filter(|(_, n)| n.disabled <= 0. && self.clear(p, n.pos))
            .filter_map(|(id, n)| {
                weapons::ray_distance_range(
                    p,
                    self.yaw,
                    n.pos,
                    weapons::CATALOG[self.pistol.kind].range,
                )
                .map(|d| (id, d))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id);
        if let Some(id) = victim {
            self.npcs[id].health -= weapons::CATALOG[self.pistol.kind].damage;
            self.drop_c4(id);
            self.notify("HIT");
        }
    }
    fn update(&mut self, r: &Renderer) {
        let forward = r.key(26) as i32 as f32 - r.key(22) as i32 as f32;
        let side = r.key(7) as i32 as f32 - r.key(4) as i32 as f32;
        self.holding = r.key(8);
        self.stance = if self.prone {
            2
        } else if self.crouched || r.key(224) {
            1
        } else {
            0
        };
        self.simulate(forward, side, r.key(225), r.key(224), r.key(44));
    }
    fn simulate(&mut self, forward: f32, side: f32, sprint: bool, crouch: bool, jump: bool) {
        if self.modal() {
            return;
        }
        self.immunity = (self.immunity - STEP).max(0.);
        if self.cuffs.active {
            if self.cuffs.tick(STEP) {
                self.arrest();
            }
            return;
        }
        self.tick_heists();
        self.bounty = self.bounty.max(self.wanted * 100);
        for lease in &mut self.save.leases {
            *lease = (*lease - STEP).max(0.);
        }
        let previous = self.player.pos;
        let forward = if self.heists.quiz || self.heists.riding.is_some() {
            0.
        } else {
            forward
        };
        let side = if self.heists.quiz || self.heists.riding.is_some() {
            0.
        } else {
            side
        };
        self.training_time = (self.training_time - STEP).max(0.);
        if self.punch_time > 0. {
            self.punch_time = (self.punch_time - STEP).max(0.);
            if self.punch_time < 0.3 && !self.punch_done {
                self.punch_done = true;
                self.punch_impact();
            }
        }
        let sprint = sprint && self.stamina > 1. && self.training_time <= 0. && self.stance == 0;
        if sprint && (forward.abs() + side.abs() > 0.) && self.driving.is_none() {
            self.stamina = (self.stamina - 22. * STEP).max(0.);
        } else {
            self.stamina =
                (self.stamina + 13. * STEP).min(100. + self.save.endurance_level as f32 * 20.);
        }
        let (forward, side, jump) = if self.training_time > 0. {
            (0., 0., false)
        } else {
            (forward, side, jump)
        };
        self.clock += STEP;
        self.pistol.tick(STEP);
        self.stun = (self.stun - STEP).max(0.);
        self.blast = (self.blast - STEP).max(0.);
        let (forward, side, jump) = if self.stun > 0. {
            (0., 0., false)
        } else {
            (forward, side, jump)
        };
        for (door, b) in self.doors.iter_mut().zip(&self.world.buildings) {
            if b.interior
                && self.npcs.iter().any(|n| {
                    n.disabled <= 0.
                        && n.pos
                            .distance(Vec3::new(b.p.x, b.p.y + 1., b.p.z - b.size.z / 2.))
                            < 5.
                })
            {
                door.open = true;
            }
            door.tick(STEP);
        }
        if let Some(t) = self.fuse {
            if t <= STEP {
                self.fuse = None;
                self.breached = true;
                self.blast = 0.7;
                if self.player.pos.distance(world::prison::WEAK) < 8. {
                    self.player.health = (self.player.health - 40.).max(1.);
                }
                self.notify("WEST WALL BREACHED - ESCAPE THROUGH THE OPENING");
            } else {
                self.fuse = Some(t - STEP);
            }
        }

        self.message_time -= STEP;
        self.shot = (self.shot - STEP).max(0.);
        if let Some(i) = self.driving {
            let world = &self.world;
            let doors = &self.doors;
            let breached = self.breached;
            let (before, rest) = self.cars.split_at_mut(i);
            let (c, after) = rest.split_first_mut().unwrap();
            let mut next = vehicle_physics::step(c, forward, -side, jump || self.stun > 0., STEP);
            let ground = self.world.surface(next.x, next.z);
            next.y = ground + 0.65;
            let hit = (1..=4).any(|j| {
                let t = j as f32 / 4.;
                let p = Vec3::new(
                    c.pos.x + (next.x - c.pos.x) * t,
                    next.y,
                    c.pos.z + (next.z - c.pos.z) * t,
                );
                [-1.5, 0., 1.5].iter().any(|offset| {
                    let q = Vec3::new(
                        p.x + c.angle.sin() * offset,
                        p.y,
                        p.z + c.angle.cos() * offset,
                    );
                    world.blocked(q, 1.05)
                        || world::prison::barrier(q, 1.05, breached)
                        || doors
                            .iter()
                            .zip(&world.buildings)
                            .any(|(d, b)| d.blocked(b, q, 1.05))
                })
            });
            let other_car = before
                .iter()
                .chain(after.iter())
                .any(|other| next.distance(other.pos) < 3.4 && (next.y - other.pos.y).abs() < 2.);
            if hit || other_car || ground < -0.3 || (next.y - c.pos.y).abs() > 0.6 {
                if c.speed.abs() > 3. {
                    c.damage = (c.damage + c.speed.abs() * 0.12).min(100.);
                }
                c.speed = 0.;
            } else {
                c.pos = next;
            }
            self.player.pos = c.pos;
            self.player.heading = c.angle;
            let delta = (std::f32::consts::PI - c.angle - self.yaw)
                .sin()
                .atan2((std::f32::consts::PI - c.angle - self.yaw).cos());
            self.yaw += delta * STEP * 2.;
            self.pitch += (0.18 - self.pitch) * STEP * 2.;
        } else {
            let length = (forward * forward + side * side).sqrt().max(1.);
            let speed = if self.stance == 2 {
                1.6
            } else if sprint {
                9. + self.save.run_level as f32 * 0.6
            } else if crouch || self.stance == 1 {
                2.5
            } else {
                6.
            };
            let dx = (self.yaw.sin() * forward + self.yaw.cos() * side) / length * speed * STEP;
            let dz = (-self.yaw.cos() * forward + self.yaw.sin() * side) / length * speed * STEP;
            let mut p = self.player.pos;
            p.x += dx;
            if !self.player_blocked(p, 0.45) {
                self.player.pos.x = p.x;
            }
            p = self.player.pos;
            p.z += dz;
            if !self.player_blocked(p, 0.45) {
                self.player.pos.z = p.z;
            }
            if dx.abs() + dz.abs() > 0. {
                let desired = dx.atan2(dz);
                let delta = (desired - self.player.heading)
                    .sin()
                    .atan2((desired - self.player.heading).cos());
                self.player.heading += delta * (STEP * 12.).min(1.);
            }
            let ground = self.world.walking_surface(self.player.pos) + 1.;
            if jump && self.stance == 0 && self.player.pos.y <= ground + 0.01 {
                self.player.vy = 7.;
            }
            self.player.vy -= 18. * STEP;
            if self.parachute && self.player.pos.y > ground + 3. {
                self.player.vy = self.player.vy.max(-3.);
            } else if self.player.pos.y <= ground + 0.1 {
                self.parachute = false;
            }
            self.player.pos.y = (self.player.pos.y + self.player.vy * STEP).max(ground);
            let ceiling = self.world.ceiling(self.player.pos) - 0.8;
            if self.player.pos.y > ceiling {
                self.player.pos.y = ceiling;
                self.player.vy = 0.;
            }
            if self.player.pos.y == ground {
                self.player.vy = 0.;
            }
            if self.player.escaped() {
                self.notify("ESCAPED - E STEAL A PARKED CAR - ROB A STORE TO BUY A PISTOL");
            }
        }
        if matches!(self.slot, 1 | 5 | 6) && self.save.weapon_owned[self.pistol.kind] {
            let goal = std::f32::consts::PI - self.yaw;
            let delta = (goal - self.player.heading)
                .sin()
                .atan2((goal - self.player.heading).cos());
            self.player.heading += delta * STEP * 12.;
        }
        if self.heists.site == Some(1) && self.heists.solved < 1 {
            if let Some(b) = self.world.buildings.iter().find(|b| b.name == "Museum") {
                if b.inside(previous) && !b.inside(self.player.pos) {
                    self.player.pos = previous;
                    self.notify("MUSEUM EXIT LOCKED - E TO SOLVE EQUATION AFTER LOOTING");
                }
            }
        }
        let moving = self.player.pos.distance(previous) > 0.001 && self.driving.is_none();
        self.movement += ((if moving { 1. } else { 0. }) - self.movement) * STEP * 12.;
        self.player.animation += STEP * (if sprint { 1.5 } else { 1. });
        self.player.pos.x = self.player.pos.x.clamp(-1000., 1000.);
        self.player.pos.z = self.player.pos.z.clamp(-1000., 1000.);
        self.alarm.collecting =
            self.holding && self.slot == 0 && self.punch_time <= 0. && self.stun <= 0.;
        self.alarm.capacity = 8 + self.save.carry_level * 2;
        match self.alarm.tick(self.player.pos, STEP) {
            Outcome::Escaped(cash) => {
                self.save.balance += cash;
                self.save.robberies += 1;
                self.wanted = 0;
                self.response = None;
                self.notify(&format!("ESCAPED WITH ${cash}"));
            }
            Outcome::Dispatch(site) => {
                let curb = self
                    .response
                    .as_ref()
                    .map(|(route, _)| *route.last().unwrap())
                    .unwrap_or_else(|| {
                        self.world
                            .roads
                            .iter()
                            .flat_map(|r| r.points.iter())
                            .min_by(|a, b| {
                                a.distance(SITES[site].center)
                                    .total_cmp(&b.distance(SITES[site].center))
                            })
                            .copied()
                            .unwrap_or(SITES[site].center)
                    });
                if self.npcs.len() > crate::npcs::population().len() {
                    self.npcs.truncate(crate::npcs::population().len());
                }
                for (j, role) in [Role::Police, Role::Police, Role::Swat].iter().enumerate() {
                    let mut p = Vec3::new(curb.x + j as f32 * 2., 1., curb.z);
                    if self.world.blocked(p, 0.45) {
                        p = curb;
                    }
                    let mut npc = Npc::new(*role, p.x, p.z);
                    npc.pos.y = self.world.walking_surface(npc.pos) + 1.;
                    npc.home.y = npc.pos.y;
                    self.npcs.push(npc);
                }
                self.response = None;
                self.notify("POLICE AND SWAT ARRIVED - TASERS AND WEAPONS");
            }
            Outcome::None => {}
        }
        if let (Some(site), Some((route, id))) = (self.alarm.site, &self.response) {
            let t = (1. - self.alarm.remaining / 30.).clamp(0., 1.) * (route.len() - 1) as f32;
            let i = (t.floor() as usize).min(route.len() - 2);
            let a = route[i];
            let b = route[i + 1];
            let f = (t - i as f32).clamp(0., 1.);
            let car = &mut self.cars[*id];
            car.pos = Vec3::new(
                a.x + (b.x - a.x) * f,
                a.y + (b.y - a.y) * f + 0.65,
                a.z + (b.z - a.z) * f,
            );
            car.angle = (b.x - a.x).atan2(b.z - a.z);
            let _ = site;
        }
        if self
            .world
            .interior(self.player.pos)
            .is_some_and(|id| self.world.buildings[id].name == "Bank")
        {
            let p = self.player.pos;
            let beam = (p.z - 20.).rem_euclid(4.);
            if p.x > 313.
                && p.x < 327.
                && (20. ..=28.).contains(&p.z)
                && !(0.25..=3.75).contains(&beam)
                && p.y < 1.3
                && self.stance == 0
            {
                self.player.health = (self.player.health - 20. * STEP).max(0.);
            }
        }
        self.tick_npcs();
    }
    fn tick_npcs(&mut self) {
        let target = self.player.pos;
        let pursuit = self.wanted > 0 && self.player.team == Team::Criminal;
        let dispatched = self.alarm.dispatched
            || self.heists.dispatched
            || (self.alarm.site.is_none()
                && self.heists.site.is_none()
                && self.wanted >= 3
                && pursuit);
        if pursuit && dispatched {
            for (door, b) in self.doors.iter_mut().zip(&self.world.buildings) {
                if b.interior
                    && self.npcs.iter().any(|n| {
                        n.officer() && n.disabled <= 0. && n.pos.distance(b.entrance()) < 3.
                    })
                {
                    door.open = true;
                }
            }
        }
        let mut taser = false;
        let mut damage = 0.;
        let mut arrested = false;
        let mut caught_robbers = Vec::new();
        let robber_targets: Vec<(usize, Vec3)> = self
            .npcs
            .iter()
            .enumerate()
            .filter(|(_, n)| n.role == Role::Robber && n.disabled <= 0.)
            .map(|(i, n)| (i, n.pos))
            .collect();
        let world = &self.world;
        let doors = &self.doors;
        let breached = self.breached;
        for (i, n) in self.npcs.iter_mut().enumerate() {
            if n.pos.distance(target) > 220. {
                continue;
            }
            let visible_suspect = n.officer()
                && pursuit
                && n.pos.distance(target) < 24.
                && (n.pos.y - target.y).abs() < 2.
                && (1..=24).all(|j| {
                    let t = j as f32 / 24.;
                    let q = Vec3::new(
                        n.pos.x + (target.x - n.pos.x) * t,
                        n.pos.y,
                        n.pos.z + (target.z - n.pos.z) * t,
                    );
                    !world.blocked(q, 0.1)
                        && !world::prison::barrier(q, 0.1, breached)
                        && !doors
                            .iter()
                            .zip(&world.buildings)
                            .any(|(d, b)| d.blocked(b, q, 0.1))
                });
            let chasing = n.officer()
                && pursuit
                && ((self.wanted >= 3 && dispatched) || visible_suspect)
                && n.pos.distance(target) < 140.;
            let report = self
                .alarm
                .site
                .filter(|_| self.alarm.dispatched)
                .map(|id| SITES[id].center)
                .or_else(|| {
                    self.heists
                        .site
                        .filter(|_| self.heists.dispatched)
                        .map(|id| {
                            let s = crate::heists::SITES[id];
                            Vec3::new(s.x, world.surface(s.x, s.z) + 1., s.z)
                        })
                });
            let investigating = n.officer() && pursuit && !chasing && report.is_some();
            let hostile = n.role == Role::Robber
                && n.disabled <= 0.
                && self.player.team != Team::Prisoner
                && n.pos.distance(target) < 18.;
            let scared = n.role == Role::Resident && pursuit && n.pos.distance(target) < 16.;
            let criminal = if n.officer() && !chasing {
                robber_targets
                    .iter()
                    .filter(|(_, p)| p.distance(n.pos) < 20.)
                    .min_by(|a, b| a.1.distance(n.pos).total_cmp(&b.1.distance(n.pos)))
                    .copied()
            } else {
                None
            };
            let goal = if n.role == Role::Vendor {
                n.home
            } else if chasing || hostile {
                target
            } else if investigating {
                report.unwrap()
            } else if let Some((_, p)) = criminal {
                p
            } else if n.role == Role::Robber && (self.clock + i as f32 * 5.) % 45. < 18. {
                SITES[i % 3].center
            } else if scared {
                Vec3::new(
                    n.pos.x + (n.pos.x - target.x) * 2.,
                    1.,
                    n.pos.z + (n.pos.z - target.z) * 2.,
                )
            } else {
                Vec3::new(
                    n.home.x
                        + (self.clock * 0.12 + i as f32).sin()
                            * if n.role == Role::Prisoner { 3. } else { 8. },
                    1.,
                    n.home.z + (self.clock * 0.12 + i as f32).cos() * 3.,
                )
            };
            let goal = if chasing && i % 3 != 0 {
                if let Some(id) = world.interior(target) {
                    let entry = world.buildings[id].entrance();
                    Vec3::new(
                        entry.x + if i % 3 == 1 { -3. } else { 3. },
                        entry.y,
                        entry.z - 1.,
                    )
                } else {
                    target
                }
            } else if !chasing
                && !investigating
                && !scared
                && (n.role == Role::Resident
                    || (n.officer() && n.home.y < world.surface(n.home.x, n.home.z) + 3.))
            {
                crate::npcs::sidewalk_goal(world, n.home, self.clock, i)
            } else {
                goal
            };
            let old = n.pos;
            n.step(
                world,
                goal,
                if chasing || investigating || criminal.is_some() {
                    6.5
                } else if scared {
                    4.
                } else {
                    1.5
                },
                STEP,
            );
            if world::prison::barrier(n.pos, 0.45, breached)
                || doors
                    .iter()
                    .zip(&world.buildings)
                    .any(|(d, b)| d.blocked(b, n.pos, 0.45))
            {
                n.pos = old;
                n.plan = 0.5;
            }
            if n.disabled > 0. {
                continue;
            }
            if let Some((id, p)) = criminal {
                if n.pos.distance(p) < 2.
                    && (n.pos.y - p.y).abs() < 1.8
                    && (1..=8).all(|j| {
                        let t = j as f32 / 8.;
                        let q = Vec3::new(
                            n.pos.x + (p.x - n.pos.x) * t,
                            n.pos.y,
                            n.pos.z + (p.z - n.pos.z) * t,
                        );
                        !world.blocked(q, 0.1)
                            && !doors
                                .iter()
                                .zip(&world.buildings)
                                .any(|(door, b)| door.blocked(b, q, 0.1))
                    })
                {
                    caught_robbers.push(id);
                }
            }
            let d = n.pos.distance(target);
            let clear = chasing
                && d < 24.
                && (n.pos.y - target.y).abs() < 1.8
                && (1..=16).all(|j| {
                    let t = j as f32 / 16.;
                    let p = Vec3::new(
                        n.pos.x + (target.x - n.pos.x) * t,
                        n.pos.y,
                        n.pos.z + (target.z - n.pos.z) * t,
                    );
                    !world.blocked(p, 0.15)
                        && !world::prison::barrier(p, 0.15, breached)
                        && !doors
                            .iter()
                            .zip(&world.buildings)
                            .any(|(door, b)| door.blocked(b, p, 0.15))
                });
            if hostile
                && d < 2.
                && n.cooldown <= 0.
                && (n.pos.y - target.y).abs() < 1.5
                && !world.blocked(
                    Vec3::new(
                        (n.pos.x + target.x) / 2.,
                        n.pos.y,
                        (n.pos.z + target.z) / 2.,
                    ),
                    0.1,
                )
            {
                damage += 8.;
                n.cooldown = 1.2;
            }
            if chasing && clear && n.cooldown <= 0. && (d >= 10. || self.immunity <= 0.) {
                if d < 10. {
                    taser = true;
                    n.cooldown = 3.;
                } else if d < 24. {
                    damage += if n.role == Role::Swat { 14. } else { 8. };
                    n.cooldown = 1.5;
                }
            }
            if chasing
                && clear
                && d < 2.5
                && self.immunity <= 0.
                && (self.driving.is_none() || self.stun > 0.)
            {
                arrested = true;
            }
        }
        for id in caught_robbers {
            self.npcs[id].disabled = 60.;
        }
        if taser && self.immunity <= 0. {
            self.stun = 2.5;
        }
        self.player.health = (self.player.health - damage).max(0.);
        if self.player.health <= 0. {
            self.arrest();
        } else if arrested && !self.cuffs.active {
            self.cuffs.start(self.wanted.clamp(1, 5), &mut self.random);
            self.notify("CUFFED - SPACE WHEN MARKER IS IN TARGET");
        }
    }
    fn draw(&mut self, r: &mut Renderer) {
        self.capture_frame += 1;
        let p = self.player.pos;
        let dist = if world::prison::underground(p) {
            2.5
        } else if self.world.interior(p).is_some() {
            3.
        } else if let Some(id) = self.driving {
            9. + self.cars[id].speed.abs() * 0.045
        } else {
            7.
        };
        let mut eye = [
            p.x - self.yaw.sin() * dist,
            p.y + 1.5 + self.pitch.sin() * dist,
            p.z + self.yaw.cos() * dist,
        ];
        let pivot = [p.x, p.y + 0.3, p.z];
        let desired = eye;
        let mut clear = pivot;
        for i in 1..=20 {
            let t = i as f32 / 20.;
            let q = Vec3::new(
                pivot[0] + (desired[0] - pivot[0]) * t,
                pivot[1] + (desired[1] - pivot[1]) * t,
                pivot[2] + (desired[2] - pivot[2]) * t,
            );
            if self.blocked(q, 0.2) || q.y > self.world.ceiling(q).min(self.world.ceiling(p)) - 0.2
            {
                break;
            }
            clear = [q.x, q.y, q.z];
        }
        eye = clear;
        // Follow lag uses render delta; resolve collisions again so smoothing cannot cross a wall.
        if self.driving.is_some() {
            let prior = self.camera.unwrap_or(eye);
            let blend = (1. - (-6. / self.stats.fps.max(30.)).exp()).clamp(0.02, 0.3);
            for i in 0..3 {
                eye[i] = prior[i] + (eye[i] - prior[i]) * blend;
            }
            eye[1] = eye[1].max(self.world.surface(eye[0], eye[2]) + 0.6);
            let mut safe = pivot;
            for i in 1..=24 {
                let t = i as f32 / 24.;
                let q = Vec3::new(
                    pivot[0] + (eye[0] - pivot[0]) * t,
                    pivot[1] + (eye[1] - pivot[1]) * t,
                    pivot[2] + (eye[2] - pivot[2]) * t,
                );
                if self.blocked(q, 0.3) {
                    break;
                }
                safe = [q.x, q.y, q.z];
            }
            eye = safe;
        }
        self.camera = Some(eye);
        let daylight = 0.55 + 0.45 * (self.clock / 120.).cos();
        let quality = crate::quality::PRESETS[self.save.quality];
        r.begin(eye, self.yaw, self.pitch, daylight);
        r.detail(quality.grain);
        if self.texture_applied != self.save.texture_level {
            r.material_resolution(self.save.texture_level);
            self.texture_applied = self.save.texture_level;
        }
        if let Some(view) = &mut self.view {
            view.update(
                r,
                p,
                crate::quality::PRESETS[self.save.quality].distance,
                self.stats.fps,
            );
            view.draw(r, eye, self.yaw, self.pitch);
        }
        if self.save.object_shadows {
            for c in self
                .world
                .colliders
                .iter()
                .filter(|c| c.size.y < 4. && c.size.y > 0.4 && c.p.distance(p) < 32.)
                .take(36)
            {
                r.object_shadow(
                    [c.p.x, c.p.y, c.p.z],
                    [c.size.x, c.size.y, c.size.z],
                    self.world.surface(c.p.x, c.p.z),
                );
            }
        }
        for b in &self.world.buildings {
            if (b.name == "Weapons" || b.name.contains("Police")) && b.p.distance(p) < 80. {
                for kind in 0..3 {
                    let xx = b.p.x - 6. + kind as f32 * 5.;
                    let zz = b.p.z + 5.;
                    r.cube([xx, b.p.y + 0.55, zz], [3., 1.1, 1.2], [0.2, 0.24, 0.3], 0.);
                    draw_weapon_model(
                        r,
                        Vec3::new(xx - 0.5, b.p.y + 1.7, zz),
                        std::f32::consts::FRAC_PI_2,
                        kind,
                    );
                    r.glass(
                        [xx, b.p.y + 1.65, zz - 0.65],
                        [3., 1.1, 0.04],
                        [0.4, 0.65, 0.72],
                        0.,
                    );
                }
            }
        }
        for id in 0..SITES.len() {
            let q = Alarm::case(id);
            if q.distance(p) > 90. {
                continue;
            }
            r.cube(
                [q.x, q.y - 0.45, q.z],
                [2.4, 1.1, 1.2],
                [0.3, 0.28, 0.23],
                0.,
            );
            let taken = self.alarm.site == Some(id) && self.alarm.loot > 0;
            if !taken {
                for i in 0..4 {
                    r.cube(
                        [q.x - 0.6 + i as f32 * 0.4, q.y + 0.16, q.z],
                        [0.3, 0.14, 0.3],
                        [0.35, 0.58, 0.29],
                        0.,
                    );
                }
            }
            if id == 3 || id == 4 {
                r.cube(
                    [q.x, q.y + 0.15, q.z],
                    [1.7, 0.6, 0.8],
                    [0.5, 0.46, 0.32],
                    0.,
                );
            }
            if self.alarm.hits[id] < 3 {
                r.glass(
                    [q.x, q.y + 0.18, q.z - 0.63],
                    [2.4, 0.9, 0.055],
                    [0.46, 0.67, 0.72],
                    0.,
                );
                r.cube(
                    [q.x, q.y + 0.68, q.z],
                    [2.4, 0.06, 1.3],
                    [0.62, 0.72, 0.75],
                    0.,
                );
                for i in 0..self.alarm.hits[id] {
                    r.bone(
                        [q.x - 0.4 + i as f32 * 0.6, q.y - 0.1, q.z - 0.67],
                        [q.x + 0.2 + i as f32 * 0.3, q.y + 0.6, q.z - 0.67],
                        0.028,
                        0.025,
                        0.,
                        [0.88, 0.92, 0.92],
                    );
                }
            } else {
                for i in 0..9 {
                    r.cube(
                        [
                            q.x - 1.2 + i as f32 * 0.3,
                            q.y - 0.96,
                            q.z - 1. + (i % 3) as f32 * 0.15,
                        ],
                        [0.18, 0.03, 0.12],
                        [0.55, 0.76, 0.8],
                        i as f32,
                    );
                }
            }
        }
        // Loot vault and destination beacon.
        r.cube([320., 1., 44.], [6., 2., 4.], GOLD, 0.);
        r.cube([-250., 3., 100.], [3., 6., 3.], [0.9, 0.22, 0.15], 0.);
        for (door, b) in self.doors.iter().zip(&self.world.buildings) {
            if b.interior && p.distance(b.p) < quality.distance {
                for side in [-1., 1.] {
                    let a = side * door.angle;
                    let cx = b.p.x + side * 1.5 - side * 0.75 * a.cos();
                    let z = b.p.z - b.size.z / 2. + side * 0.75 * a.sin();
                    r.cube([cx, b.p.y + 1.5, z], [1.5, 3., 0.18], [0.28, 0.35, 0.4], a);
                    r.glass(
                        [cx - 0.11 * a.sin(), b.p.y + 2., z - 0.11 * a.cos()],
                        [0.9, 0.8, 0.04],
                        [0.3, 0.48, 0.56],
                        a,
                    );
                }
            }
        }
        if self.blast > 0. {
            r.rounded(
                [-170., 2.5, -110.],
                [5., 5., 5.],
                [1., 0.5 + self.blast * 0.5, 0.12],
                0.,
                12,
                1.,
            );
        }
        if !self.breached {
            r.cube([-170., 3., -110.], [1., 6., 16.], [0.48, 0.42, 0.37], 0.);
        }
        if !self.breached {
            for z in [-115., -110., -106.] {
                r.cube([-169.4, 2., z], [0.05, 0.12, 3.], [0.2, 0.15, 0.12], 0.);
            }
        }
        if !self.charge_taken {
            r.cube([-140., 0.5, -135.], [0.7, 0.7, 0.7], [0.72, 0.24, 0.12], 0.);
        }
        let car_start = r.triangles;
        for c in &self.cars {
            let d = p.distance(c.pos);
            if d > quality.distance
                || !crate::world_view::sphere_visible(
                    [c.pos.x, c.pos.y, c.pos.z],
                    4.,
                    eye,
                    self.yaw,
                    self.pitch,
                    r.width as f32 / r.height as f32,
                    quality.distance,
                )
            {
                continue;
            }
            if self.save.object_shadows && d < quality.detail {
                r.rounded(
                    [
                        c.pos.x,
                        self.world.surface(c.pos.x, c.pos.z) + 0.06,
                        c.pos.z,
                    ],
                    [2.5, 0.03, 4.7],
                    [0.12, 0.15, 0.12],
                    c.angle,
                    10,
                    1.,
                );
            }
            let lod = usize::from(d < quality.detail);
            draw_car(r, c, &self.car_models[c.kind][lod], lod);
        }
        self.vehicle_triangles = r.triangles - car_start;
        if self.driving.is_none() {
            draw_person(
                r,
                p,
                self.player.heading,
                match self.player.team {
                    Team::Prisoner => [1., 0.4, 0.1],
                    Team::Criminal => [0.85, 0.17, 0.2],
                    Team::Police => [0.18, 0.3, 0.8],
                },
                self.player.animation,
                true,
                quality.segments,
                self.movement,
                if self.punch_time > 0. {
                    ((0.55 - self.punch_time) / 0.55 * std::f32::consts::PI).sin()
                } else if self.alarm.collecting && self.alarm.site.is_some() {
                    0.65 + 0.2 * (self.clock * 12.).sin()
                } else {
                    0.
                },
                self.training_time,
                self.stance,
            );
            if self.alarm.site.is_some() && self.alarm.progress > 0.2 {
                let a = self.player.heading;
                let prone = self.stance == 2;
                let yy = p.y - if self.stance > 0 { 0.45 } else { 0.1 };
                r.rounded(
                    [p.x - a.sin() * 0.3, yy, p.z - a.cos() * 0.3],
                    [0.55, if prone { 0.18 } else { 0.65 }, 0.3],
                    [0.44, 0.32, 0.18],
                    a,
                    10,
                    0.22,
                );
                if !prone {
                    for side in [-1., 1.] {
                        r.bone(
                            [
                                p.x + side * 0.19 * a.cos() - a.sin() * 0.46,
                                yy + 0.28,
                                p.z - side * 0.19 * a.sin() - a.cos() * 0.46,
                            ],
                            [
                                p.x + side * 0.19 * a.cos() - a.sin() * 0.46,
                                yy - 0.28,
                                p.z - side * 0.19 * a.sin() - a.cos() * 0.46,
                            ],
                            0.055,
                            0.03,
                            a,
                            [0.2, 0.18, 0.13],
                        );
                    }
                }
            }
            if matches!(self.slot, 1 | 5 | 6) && self.save.weapon_owned[self.pistol.kind] {
                draw_weapon_model(r, p, self.player.heading, self.pistol.kind);
            }
            if self.slot == 2 && self.explosive {
                let a = self.player.heading;
                r.rounded(
                    [p.x + 0.36 * a.cos(), p.y - 0.47, p.z - 0.36 * a.sin()],
                    [0.25, 0.25, 0.3],
                    [0.3, 0.36, 0.16],
                    a,
                    8,
                    0.3,
                );
            }
        }
        let npc_start = r.triangles;
        for n in &self.npcs {
            if n.disabled <= 0.
                && p.distance(n.pos) < quality.distance
                && crate::world_view::sphere_visible(
                    [n.pos.x, n.pos.y, n.pos.z],
                    2.,
                    eye,
                    self.yaw,
                    self.pitch,
                    r.width as f32 / r.height as f32,
                    quality.distance,
                )
            {
                if self.save.object_shadows && p.distance(n.pos) < quality.detail {
                    r.rounded(
                        [n.pos.x, self.world.walking_surface(n.pos) + 0.05, n.pos.z],
                        [0.85, 0.025, 0.65],
                        [0.15, 0.18, 0.14],
                        0.,
                        8,
                        1.,
                    );
                }
                draw_person(
                    r,
                    n.pos,
                    n.heading,
                    n.color(),
                    n.phase,
                    p.distance(n.pos) < quality.detail,
                    quality.segments,
                    n.motion,
                    if n.officer() && self.wanted > 0 {
                        0.35
                    } else {
                        0.
                    },
                    0.,
                    0,
                );
                if n.officer() || n.role == Role::Robber {
                    draw_weapon(r, n.pos, n.heading, n.role == Role::Swat);
                }
                if n.officer() {
                    r.cube(
                        [
                            n.pos.x + 0.35 * n.heading.cos(),
                            n.pos.y - 0.12,
                            n.pos.z - 0.35 * n.heading.sin(),
                        ],
                        [0.1, 0.13, 0.2],
                        [0.85, 0.75, 0.1],
                        n.heading,
                    );
                }
                if n.cooldown > 1.35 && n.pos.distance(p) < 25. {
                    r.rounded(
                        [
                            n.pos.x + n.heading.sin() * 0.7,
                            n.pos.y,
                            n.pos.z + n.heading.cos() * 0.7,
                        ],
                        [0.2, 0.15, 0.2],
                        [0.7, 0.85, 1.],
                        0.,
                        6,
                        1.,
                    );
                }
                if n.role == Role::Police {
                    r.rounded(
                        [n.pos.x, n.pos.y + 0.71, n.pos.z],
                        [0.42, 0.14, 0.43],
                        [0.1, 0.2, 0.35],
                        n.heading,
                        quality.segments,
                        0.4,
                    );
                }
                if n.role == Role::Swat {
                    r.rounded(
                        [n.pos.x, n.pos.y + 0.69, n.pos.z],
                        [0.43, 0.25, 0.44],
                        [0.1, 0.13, 0.15],
                        n.heading,
                        quality.segments,
                        0.7,
                    );
                }
            }
        }
        if self.shot > 0. {
            r.cube(
                [
                    p.x + self.yaw.sin() * 5.,
                    p.y + 1.,
                    p.z - self.yaw.cos() * 5.,
                ],
                [0.2, 0.2, 2.],
                [1., 0.9, 0.2],
                -self.yaw,
            );
        }
        self.draw_heists(r);
        let scene_draws = r.draws;
        self.npc_triangles = r.triangles - npc_start;
        let scene_triangles = r.triangles;
        self.scene_triangles = scene_triangles;
        r.ui();
        self.draw_interface(r);
        self.draw_escape(r);
        self.draw_heist_ui(r);
        if self.debug
            && !self.garage
            && !self.menu
            && !self.map
            && self.workshop.is_none()
            && !self.training
        {
            r.rect(350., 92., 445., 90., [0.04, 0.07, 0.12]);
            text(
                r,
                360.,
                104.,
                1.3,
                &format!(
                    "FPS {:.0}  FRAME {:.1}MS  SCENE DRAWS {}",
                    self.stats.fps,
                    1000. / self.stats.fps.max(1.),
                    scene_draws
                ),
                WHITE,
            );
            text(
                r,
                360.,
                128.,
                1.2,
                &format!(
                    "CARS {} NPC {} FIXED PHYSICS 60 HZ",
                    self.cars.len(),
                    self.npcs.len()
                ),
                WHITE,
            );
            text(r, 360., 151., 1., &r.driver.clone(), GOLD);
            if let Some(v) = &self.view {
                text(
                    r,
                    360.,
                    170.,
                    1.2,
                    &format!(
                        "CHUNKS {} / {} TRI {} GPU {:.1}MB",
                        v.visible,
                        v.active(),
                        scene_triangles,
                        v.bytes as f32 / 1048576.
                    ),
                    WHITE,
                );
            }
        }
        if self.debug
            && !self.garage
            && !self.menu
            && !self.map
            && self.workshop.is_none()
            && !self.training
        {
            text(
                r,
                360.,
                188.,
                1.1,
                &format!(
                    "RAM {:.0}MB UPDATE {:.2}MS RENDER {:.1}MS",
                    self.stats.rss_mb, self.stats.update_ms, self.stats.render_ms
                ),
                WHITE,
            );
        }
        if let Ok(path) = std::env::var("JAILBREAK_CAPTURE") {
            if !self.captured && self.clock >= self.capture_at && self.capture_frame >= 30 {
                self.captured = true;
                if let Err(e) = r.capture_ppm(&path) {
                    eprintln!("Capture failed: {e}");
                }
            }
        }
        r.end();
        self.stats.frame();
    }
    fn draw_map(&self, r: &mut Renderer, rect: [f32; 4]) {
        r.rect(rect[0], rect[1], rect[2], rect[3], [0.14, 0.27, 0.21]);
        if let Some(view) = &self.view {
            r.texture_rect(view.map, rect);
        }
        for poi in &self.world.pois {
            let q = minimap::project(poi.p.x, poi.p.z, rect);
            r.rect(q[0] - 2., q[1] - 2., 4., 4., poi.color);
            if self.map
                && rect[2] > 300.
                && matches!(
                    poi.name,
                    "Bank"
                        | "Weapons"
                        | "Garage"
                        | "Prison"
                        | "Criminal Base"
                        | "Gym"
                        | "Safehouse"
                        | "Airport"
                        | "Industrial"
                        | "Jewelry"
                        | "Casino"
                        | "Museum"
                        | "Power Plant"
                        | "Gas Station"
                        | "Donut Shop"
                        | "Grocery"
                )
            {
                let offset = match poi.name {
                    "Safehouse" => 12.,
                    "Gym" => 25.,
                    _ => 0.,
                };
                text(r, q[0] + 5., q[1] + offset, 0.9, poi.name, WHITE);
            }
        }
        for n in &self.npcs {
            let q = minimap::project(n.pos.x, n.pos.z, rect);
            r.rect(q[0], q[1], 3., 3., [0.2, 0.4, 1.]);
        }
        if let Some(id) = self.alarm.site {
            let site = SITES[id];
            for i in 0..48 {
                let a = i as f32 * std::f32::consts::TAU / 48.;
                let q = minimap::project(
                    site.center.x + a.sin() * site.radius,
                    site.center.z + a.cos() * site.radius,
                    rect,
                );
                r.rect(q[0], q[1], 2., 2., [1., 0.15, 0.1]);
            }
        }
        if let Some(w) = self.waypoint {
            let q = minimap::project(w.x, w.z, rect);
            r.rect(q[0] - 3., q[1] - 3., 6., 6., [0.4, 1., 0.8]);
        }
        let tip = minimap::project(
            self.player.pos.x + self.player.heading.sin() * 35.,
            self.player.pos.z + self.player.heading.cos() * 35.,
            rect,
        );
        r.rect(tip[0] - 1., tip[1] - 1., 2., 2., GOLD);
        let q = minimap::project(self.player.pos.x, self.player.pos.z, rect);
        r.rect(q[0] - 3., q[1] - 3., 6., 6., WHITE);
    }
}
#[allow(clippy::too_many_arguments)] // Render primitive parameters match the renderer API.
fn part(
    r: &mut Renderer,
    p: [f32; 3],
    size: [f32; 3],
    color: [f32; 3],
    a: f32,
    detail: bool,
    n: usize,
    round: f32,
) {
    if detail {
        r.rounded(p, size, color, a, n, round);
    } else {
        r.cube(p, size, color, a);
    }
}
#[allow(clippy::too_many_arguments)]
fn draw_person(
    r: &mut Renderer,
    p: Vec3,
    a: f32,
    c: [f32; 3],
    t: f32,
    detail: bool,
    n: usize,
    motion: f32,
    action: f32,
    training: f32,
    stance: u8,
) {
    if stance == 2 {
        draw_crawler(r, p, a, c, t, motion);
        return;
    }
    let p = Vec3::new(p.x, p.y - if stance == 1 { 0.35 } else { 0. }, p.z);
    let bob = (t * 18.).cos() * 0.025 * motion;
    let local = |x: f32, y: f32, z: f32| {
        [
            p.x + x * a.cos() + z * a.sin(),
            p.y + y + bob,
            p.z - x * a.sin() + z * a.cos(),
        ]
    };
    // Original block avatar: flat faces, slight bevels, articulated elbows and knees.
    part(
        r,
        local(0., 0., 0.),
        [0.74, 0.8, 0.38],
        c,
        a,
        detail,
        n,
        0.16,
    );
    part(
        r,
        local(0., 0.68, 0.),
        [0.48, 0.49, 0.46],
        [0.76, 0.59, 0.42],
        a,
        detail,
        n,
        0.18,
    );
    if detail {
        part(
            r,
            local(0., 0.95, -0.02),
            [0.5, 0.14, 0.47],
            [0.18, 0.14, 0.1],
            a,
            true,
            n,
            0.12,
        );
        r.cube(
            local(0., -0.32, 0.2),
            [0.74, 0.08, 0.025],
            [0.12, 0.14, 0.15],
            a,
        );
        r.cube(
            local(0.2, 0.18, 0.2),
            [0.18, 0.16, 0.025],
            [c[0] * 0.7, c[1] * 0.7, c[2] * 0.7],
            a,
        );
        for x in [-0.1, 0.1] {
            r.cube(
                local(x, 0.72, 0.24),
                [0.045, 0.055, 0.025],
                [0.08, 0.07, 0.06],
                a,
            );
        }
        r.cube(
            local(0., 0.59, 0.241),
            [0.13, 0.023, 0.02],
            [0.24, 0.15, 0.11],
            a,
        );
    }
    let squat = if training > 0. {
        (training * 5.).sin().abs() * 0.18
    } else {
        0.
    };
    for side in [-1., 1.] {
        let swing = (t * 9.).sin() * 0.42 * side * motion;
        let hip = local(side * 0.19, -0.4 - squat, 0.);
        let knee = local(
            side * 0.19,
            -0.69 - squat + if stance == 1 { 0.25 } else { 0. },
            swing * 0.35 + if stance == 1 { 0.25 } else { 0. },
        );
        let foot = local(
            side * 0.19,
            -0.96 + if stance == 1 { 0.35 } else { 0. },
            swing * 0.6,
        );
        r.bone(hip, knee, 0.29, 0.3, a, [0.19, 0.22, 0.27]);
        r.bone(knee, foot, 0.27, 0.28, a, [0.16, 0.19, 0.24]);
        r.cube(
            [foot[0] + 0.04 * a.sin(), foot[1], foot[2] + 0.04 * a.cos()],
            [0.29, 0.13, 0.4],
            [0.08, 0.1, 0.12],
            a,
        );
        let reach = if side > 0. {
            action * 0.72
        } else {
            action * 0.25
        };
        let shoulder = local(side * 0.51, 0.28 - squat, 0.);
        let elbow = local(
            side * 0.5,
            -0.04 - squat + reach * 0.28,
            -swing * 0.16 + reach * 0.35,
        );
        let hand = local(
            side * 0.5,
            -0.4 - squat + reach * 0.5,
            -swing * 0.28 + reach,
        );
        r.bone(shoulder, elbow, 0.26, 0.28, a, c);
        r.bone(elbow, hand, 0.24, 0.25, a, c);
        part(
            r,
            hand,
            [0.24, 0.22, 0.25],
            [0.76, 0.59, 0.42],
            a,
            detail,
            n,
            0.18,
        );
    }
}
fn draw_weapon(r: &mut Renderer, p: Vec3, a: f32, swat: bool) {
    draw_weapon_model(r, p, a, if swat { 2 } else { 0 });
}
fn draw_car(r: &mut Renderer, c: &Car, model: &vehicles::body::Model, lod: usize) {
    let p = [c.pos.x, c.pos.y, c.pos.z];
    let key = c.kind * 4 + lod * 2;
    r.cached_solid_model(key, &model.paint, p, c.angle, c.color);
    r.cached_solid_model(key + 1, &model.trim, p, c.angle, [1.; 3]);
}
pub fn run() -> Result<(), String> {
    if std::env::args().any(|a| a == "--credits") {
        println!("{}", include_str!("../../vehicles/assets/ATTRIBUTION.md"));
        return Ok(());
    }
    if std::env::args().any(|a| a == "--audit-map") {
        let w = World::new();
        w.audit()?;
        let length = w
            .roads
            .iter()
            .flat_map(|r| r.points.windows(2))
            .map(|s| s[0].distance(s[1]))
            .sum::<f32>();
        println!("MAP AUDIT OK: extent=2048m buildings={} interiors={} roads={} road_length={:.0}m colliders={} trees={} POIs={} movable_metadata={}",w.buildings.len(),w.buildings.iter().filter(|b|b.interior).count(),w.roads.len(),length,w.colliders.len(),w.trees.len(),w.pois.len(),w.colliders.iter().filter(|c|c.movable).count());
        return Ok(());
    }

    let mut g = if std::env::args().any(|x| x == "--smoke-test") {
        Game::from_save(Default::default())?
    } else {
        Game::new()?
    };
    if let Ok(value) = std::env::var("JAILBREAK_QUALITY") {
        g.save.quality = value.parse::<usize>().unwrap_or(1).min(3);
    }
    if let Ok(v) = std::env::var("JAILBREAK_TEXTURE_LEVEL") {
        g.save.texture_level = v.parse::<usize>().unwrap_or(0).min(3);
    }
    if let Ok(v) = std::env::var("JAILBREAK_SHADOWS") {
        g.save.object_shadows = v != "0";
    }
    let mut r = Renderer::new()?;
    println!("OpenGL {} renderer: {}", r.version(), r.driver);
    g.world.audit()?;
    g.view = Some(crate::world_view::WorldView::new(
        g.world.clone(),
        &mut r,
        g.player.pos,
        crate::quality::PRESETS[g.save.quality].distance,
    ));
    let smoke = std::env::args().any(|x| x == "--smoke-test");
    if let Ok(scene) = std::env::var("JAILBREAK_SCENE") {
        let (x, z, yaw) = match scene.as_str() {
            "museum-new" => (-200., -640., std::f32::consts::PI),
            "power-new" => (720., -179., std::f32::consts::PI),
            "gas-new" => (-650., 32., std::f32::consts::PI),
            "jstore-front-new" => (247., 158., 2.75),
            "jstore-showroom-new" | "jstore-loot-new" | "jstore-lasers-new" | "jstore-roof-new" => {
                (260., 181., std::f32::consts::PI)
            }
            "jstore-stairs-new" => (249., 178., std::f32::consts::PI),
            "jewelry-new" => (260., 180., std::f32::consts::PI),
            "casino-new" => (375., 235., std::f32::consts::PI),
            "flats-new" | "apartment-new" => (110., 93., std::f32::consts::PI),
            "quiz-new" | "cuffs-new" | "customs-new" => (-95., -83., std::f32::consts::PI),
            "train-new" => (800., -12., std::f32::consts::PI / 2.),
            "armory" => (60., 240., std::f32::consts::PI),
            "housing" => (-220., 210., std::f32::consts::PI),
            "factory-bag" => (640., 215., std::f32::consts::PI),
            "penthouse" => (350., -52., std::f32::consts::PI),
            "crawl" | "crouch" => (-100., -160., std::f32::consts::PI),
            "menu" => (-95., -83., std::f32::consts::PI),
            "weapon" => (60., 240., std::f32::consts::PI),
            "dispatch" => (110., 240., std::f32::consts::PI),
            "donut" | "alarm" => (110., 230., std::f32::consts::PI),
            "gym" => (-260., 335., std::f32::consts::PI),
            "safehouse" => (-335., 315., std::f32::consts::PI),
            "factory" => (570., 165., std::f32::consts::PI),
            "garage" => (430., -60., std::f32::consts::PI),
            "driving" => (-95., -76., std::f32::consts::PI),
            "map" => (210., 230., std::f32::consts::PI),
            "grocery" => (214.5, 239.5, std::f32::consts::PI),
            "car" => (-95., -83., std::f32::consts::PI),
            "hills" => (-250., 590., std::f32::consts::PI),
            "sewer" => (-116., -138., std::f32::consts::PI),
            "cells" => (-145., -194., std::f32::consts::PI),
            "prison-police" => (-55., -141., std::f32::consts::PI),
            "city" => (310., -20., std::f32::consts::PI),
            "harbor" => (825., 110., 1.8),
            "mountains" => (200., -580., 0.),
            "airport" => (720., 660., std::f32::consts::PI),
            "bridge" => (-455., 0., 2.),
            "interior" => (320., 30., std::f32::consts::PI),
            _ => (-100., -160., 0.),
        };
        g.player.pos = Vec3::new(
            x,
            if scene == "sewer" {
                -3.
            } else {
                g.world.surface(x, z) + 1.
            },
            z,
        );
        g.yaw = yaw;
        g.menu = false;
        if matches!(
            scene.as_str(),
            "cells"
                | "gym"
                | "safehouse"
                | "garage"
                | "prison-police"
                | "donut"
                | "grocery"
                | "interior"
                | "alarm"
                | "weapon"
                | "dispatch"
        ) {
            for door in &mut g.doors {
                door.open = true;
                door.angle = std::f32::consts::FRAC_PI_2;
            }
        }
        if scene.ends_with("-new") {
            for d in &mut g.doors {
                d.open = true;
                d.angle = std::f32::consts::FRAC_PI_2;
            }
        }
        if smoke
            && matches!(
                scene.as_str(),
                "vehicle-new" | "vehicle-paint-new" | "vehicle-lod-new"
            )
        {
            let kind = std::env::var("JAILBREAK_VEHICLE")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0)
                .min(CATALOG.len() - 1);
            let pos = Vec3::new(720., g.world.surface(720., 660.) + 0.65, 660.);
            g.cars = vec![Car::new(kind, pos)];
            g.player.pos = pos;
            g.driving = Some(0);
            g.yaw = -0.65;
            g.pitch = 0.18;
            if scene == "vehicle-paint-new" {
                g.cars[0].color = [0.82, 0.12, 0.08];
            }
            if scene == "vehicle-lod-new" {
                g.driving = None;
                g.cars[0].pos.z -= 85.;
                g.cars[0].pos.y = g.world.surface(g.cars[0].pos.x, g.cars[0].pos.z) + 0.65;
                g.yaw = 0.;
                g.pitch = 0.;
            }
        }
        if smoke && scene.starts_with("jstore-") {
            g.pitch = if matches!(scene.as_str(), "jstore-front-new" | "jstore-stairs-new") {
                -0.08
            } else {
                0.08
            };
            let floor = if scene == "jstore-loot-new" {
                4.
            } else if scene == "jstore-lasers-new" {
                8.
            } else if scene == "jstore-roof-new" {
                12.
            } else if scene == "jstore-stairs-new" {
                4.
            } else {
                0.
            };
            g.player.pos.y = g.world.surface(260., 190.) + floor + 1.;
            if scene != "jstore-front-new" {
                g.heists.start(0);
                if scene != "jstore-showroom-new" {
                    g.heists.strikes = 3;
                }
                if matches!(scene.as_str(), "jstore-lasers-new" | "jstore-roof-new") {
                    g.heists.bag = 100;
                }
            }
        }
        if smoke && matches!(scene.as_str(), "kitchen-new" | "bathroom-new") {
            let b = g
                .world
                .buildings
                .iter()
                .find(|b| b.name == "Office" && b.p.x == 110. && b.p.z == 100.)
                .ok_or("Apartment showcase building missing")?;
            g.player.pos = if scene == "kitchen-new" {
                Vec3::new(b.p.x + b.size.x / 2. - 4., b.p.y + 5., b.p.z - 1.)
            } else {
                Vec3::new(
                    b.p.x + b.size.x / 2. - 4.,
                    b.p.y + 5.,
                    b.p.z + b.size.z / 2. - 4.5,
                )
            };
            g.yaw = if scene == "kitchen-new" {
                std::f32::consts::FRAC_PI_2
            } else {
                std::f32::consts::PI
            };
            g.pitch = 0.05;
        }
        if smoke && scene == "skyline-new" {
            g.player.pos = Vec3::new(260., 36., -90.);
            g.yaw = std::f32::consts::PI;
            g.pitch = 0.08;
        }
        if smoke && scene == "locomotive-new" {
            let (p, a) = crate::heists::Heists::train(g.clock, 0);
            g.player.pos = Vec3::new(p.x + a.sin() * 10., p.y, p.z + a.cos() * 10.);
            g.yaw = -0.65;
            g.pitch = 0.2;
        }
        if scene == "apartment-new" {
            g.player.pos.x += 5.;
            g.player.pos.y += 4.;
            g.player.pos.z += 7.;
        }
        if scene == "quiz-new" {
            g.heists.start(2);
            g.heists.question();
        }
        if scene == "cuffs-new" {
            g.cuffs.start(5, &mut g.random);
        }
        if scene == "customs-new" {
            g.workshop = Some(0);
            g.workshop_config = true;
            g.save.balance = 2500;
        }
        if scene == "armory" {
            g.armory = true;
            g.save.balance = 900;
            g.weapon_selection = 1;
        }
        if scene == "housing" {
            g.housing = Some(0);
            g.save.balance = 200;
        }
        if scene == "penthouse" {
            g.player.pos.y = 19.;
        }
        if scene == "crawl" {
            g.prone = true;
            g.stance = 2;
        }
        if scene == "crouch" {
            g.crouched = true;
            g.stance = 1;
        }
        if scene == "factory-bag" {
            g.alarm.start(3);
            g.alarm.hits[3] = 3;
            g.alarm.progress = 4.;
            g.player.team = Team::Criminal;
            g.wanted = 3;
        }
        if scene == "weapon" {
            g.save.balance = 350;
            g.buy_pistol();
            g.player.team = Team::Criminal;
        }
        if scene == "dispatch" {
            g.player.pos = SITES[1].center;
            g.alarm.start(1);
            g.player.team = Team::Criminal;
            g.wanted = 3;
            g.prepare_response(1);
            g.alarm.hits[1] = 3;
            g.alarm.remaining = 3.5;
            g.alarm.progress = 3.;
        }
        if scene == "alarm" {
            g.player.pos = Vec3::new(110., 1., 234.);
            g.alarm.start(1);
            g.alarm.hits[1] = 2;
            g.player.team = Team::Criminal;
            g.wanted = 3;
            g.prepare_response(1);
        }
        if scene == "map" {
            g.map = true;
        }
        if scene == "service" {
            g.player.pos = Vec3::new(430., 1., -70.);
            g.cars[0].pos = g.player.pos;
            g.workshop = Some(0);
        }
        if scene == "training" {
            g.training = true;
        }
        if scene == "driving" {
            g.driving = Some(0);
            g.player.pos = g.cars[0].pos;
            g.cars[0].speed = 15.;
        }
        if scene == "graphics" {
            g.garage = true;
        }
    }

    if smoke {
        g.menu = std::env::var("JAILBREAK_SCENE").as_deref() == Ok("menu");
        g.debug = std::env::var("JAILBREAK_DEBUG").as_deref() != Ok("0");
    }
    let mut last = std::time::Instant::now();
    let mut accumulator = 0.;
    if std::env::var("JAILBREAK_TIME").as_deref() == Ok("night") {
        g.clock = std::f32::consts::PI * 120.;
    }
    g.capture_at =
        if g.modal() || std::env::var("JAILBREAK_SCENE").is_ok_and(|s| s.ends_with("-new")) {
            g.clock
        } else {
            g.clock + 3.
        };
    let frame_limit = std::env::var("JAILBREAK_FRAMES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(480)
        .clamp(60, 2000);
    let mut timings = Vec::with_capacity(if smoke { frame_limit } else { 0 });
    let tour = smoke && std::env::var("JAILBREAK_SCENE").as_deref() == Ok("tour");
    let mut frames = 0;
    let mut running = true;
    while running {
        let frame_start = std::time::Instant::now();
        if tour {
            let x = (-520. + frames as f32 * 2.5).min(840.);
            g.player.pos = Vec3::new(x, g.world.surface(x, 0.) + 1., 0.);
            g.player.team = Team::Criminal;
            g.yaw = std::f32::consts::FRAC_PI_2;
        }

        while let Some(e) = r.poll() {
            match e.kind() {
                0x100 => running = false,
                0x400 => {
                    if !g.modal() && !g.cursor {
                        let (x, y) = e.motion();
                        g.yaw += x as f32 * g.save.sensitivity;
                        g.pitch = (g.pitch + y as f32 * g.save.sensitivity).clamp(-0.1, 0.9);
                    }
                }
                0x401 => {
                    let (x, y) = e.click();
                    let x = x as f32 * 1024. / r.width as f32;
                    let y = y as f32 * 576. / r.height as f32;
                    if g.modal() || g.cursor {
                        g.ui_click(x, y);
                    } else {
                        g.fire();
                    }
                    r.capture(!g.modal() && !g.cursor);
                }
                0x300 if !e.repeat() => {
                    let k = e.key();
                    running = g.key_action(k);
                    r.capture(!g.modal() && !g.cursor);
                }
                _ => {}
            }
        }
        let now = std::time::Instant::now();
        accumulator += (now - last).as_secs_f32().min(0.1);
        last = now;
        let update_start = std::time::Instant::now();
        while accumulator >= STEP {
            g.update(&r);
            accumulator -= STEP;
        }
        g.stats.update_ms = update_start.elapsed().as_secs_f32() * 1000.;
        let render_start = std::time::Instant::now();
        if smoke && std::env::var("JAILBREAK_SCENE").as_deref() == Ok("skyline-new") {
            g.player.pos = Vec3::new(260., 36., -90.);
            g.player.vy = 0.;
        }
        g.draw(&mut r);
        g.stats.render_ms = render_start.elapsed().as_secs_f32() * 1000.;
        if smoke {
            timings.push(frame_start.elapsed().as_secs_f32() * 1000.);
        }
        if smoke {
            r.validate()?;
        }
        frames += 1;
        if smoke && frames >= frame_limit {
            running = false;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    r.validate()?;
    if smoke {
        let (parts, bytes) = r.cached_model_usage();
        println!("VEHICLE_CACHE parts={parts} bytes={bytes}");
        timings.sort_by(|a, b| a.total_cmp(b));
        let mean = timings.iter().sum::<f32>() / timings.len() as f32;
        let p95 = timings[(timings.len() as f32 * 0.95) as usize];
        let maximum = timings.last().copied().unwrap_or(0.);
        if let Some(v) = &g.view {
            println!(
                "GEOMETRY scene_triangles={} world_triangles={} interior_triangles={} quality={} vehicles={} actors_and_effects={}",
                g.scene_triangles, v.submitted_triangles, v.interior_triangles, g.save.quality, g.vehicle_triangles, g.npc_triangles
            );
            println!("PROFILE scene={} frames={} mean_ms={:.2} p95_ms={:.2} max_ms={:.2} rss_mb={:.1} gpu_mb={:.1} active_chunks={} visible_chunks={} effective_distance={:.0}",std::env::var("JAILBREAK_SCENE").unwrap_or_else(|_|"prison".into()),frames,mean,p95,maximum,profiling::rss_mb(),v.bytes as f32/1048576.,v.active(),v.visible,v.effective_distance);
        }
    }
    g.view.take();
    if !smoke {
        g.save();
    }
    println!(
        "Rendered {frames} frames; observed {:.1} FPS (host only)",
        g.stats.fps
    );
    Ok(())
}

fn configure_car(c: &mut Car, s: &save_system::Save) {
    c.engine_level = if s.car_engine_selected[c.kind] == u32::MAX {
        s.car_engine[c.kind]
    } else {
        s.car_engine_selected[c.kind]
    };
    c.brake_level = s.car_brakes[c.kind];
    c.suspension_level = s.car_suspension[c.kind];
    if s.car_paint[c.kind] != [0.; 3] {
        c.color = s.car_paint[c.kind];
    }
}

include!("interface.rs");
include!("expansion.rs");

#[cfg(test)]
mod tests {
    use super::*;
    fn break_case(g: &mut Game, id: usize) {
        let case = Alarm::case(id);
        g.player.pos = Vec3::new(case.x, case.y, case.z - 2.);
        g.yaw = std::f32::consts::PI;
        g.slot = 0;
        for _ in 0..3 {
            g.fire();
            for _ in 0..36 {
                g.simulate(0., 0., false, false, false);
            }
        }
        g.holding = true;
    }

    #[test]
    fn sewer_escape_walks_below_perimeter_and_uses_exit_ladder() {
        let mut g = Game::from_save(save_system::Save::default()).unwrap();
        g.menu = false;
        g.player = Player::new(Team::Prisoner);
        g.player.pos = world::prison::HATCH;
        g.interact();
        assert_eq!(g.player.pos.y, -3.);
        g.npcs.clear();
        for (x, z) in world::prison::ROUTE.into_iter().skip(1) {
            for _ in 0..700 {
                let dx = x - g.player.pos.x;
                let dz = z - g.player.pos.z;
                if dx.abs() + dz.abs() < 0.12 {
                    break;
                }
                g.yaw = dx.atan2(-dz);
                g.simulate(1., 0., false, false, false);
            }
        }
        assert!((g.player.pos.z + 78.).abs() < 0.2);
        assert_eq!(g.player.team, Team::Prisoner);
        assert!((g.player.pos.y + 3.).abs() < 0.01);
        g.interact();
        assert_eq!(g.player.pos.y, 1.);
        g.simulate(0., 0., false, false, false);
        assert_eq!(g.player.team, Team::Criminal);
    }
    #[test]
    fn sewer_side_walls_prevent_surface_teleport() {
        let mut g = Game::from_save(save_system::Save::default()).unwrap();
        g.menu = false;
        g.player.pos = Vec3::new(-116., -3., -140.);
        g.yaw = 0.;
        for _ in 0..120 {
            g.simulate(0., 1., false, false, false);
        }
        assert!((g.player.pos.x + 116.).abs() < 2.3);
        assert_eq!(g.player.pos.y, -3.);
    }
    #[test]
    fn car_cannot_drive_through_cell_block() {
        let mut g = Game::from_save(save_system::Save::default()).unwrap();
        g.menu = false;
        g.player = Player::new(Team::Criminal);
        g.cars[0].pos = Vec3::new(-125., 0.65, -183.);
        g.cars[0].angle = -std::f32::consts::FRAC_PI_2;
        g.driving = Some(0);
        for _ in 0..180 {
            g.simulate(1., 0., false, false, false);
        }
        assert!(g.cars[0].pos.x > -129.5);
    }
    #[test]
    fn parked_car_blocks_driving() {
        let mut g = Game::from_save(save_system::Save::default()).unwrap();
        g.menu = false;
        g.player = Player::new(Team::Criminal);
        g.driving = Some(0);
        g.cars[0].pos = Vec3::new(-95., 0.65, -76.);
        g.cars[1].pos = Vec3::new(-95., 0.65, -66.);
        for _ in 0..180 {
            g.simulate(1., 0., false, false, false);
        }
        assert!(g.cars[0].pos.z < -69.3);
        assert_eq!(g.cars[1].pos.z, -66.);
    }
    #[test]
    fn closed_gate_and_bomb_escape_are_real_collisions() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.player.pos = Vec3::new(-95., 1., -104.);
        g.yaw = std::f32::consts::PI;
        for _ in 0..80 {
            g.simulate(1., 0., false, false, false);
        }
        assert!(g.player.pos.z < -100.5);
        assert_eq!(g.player.team, Team::Prisoner);
        g.player.pos = world::prison::CHARGE;
        g.interact();
        assert!(g.explosive);
        assert_eq!(g.slot, 2);
        g.player.pos = Vec3::new(-167., 1., -110.);
        assert!(g.blocked(world::prison::WEAK, 0.45));
        g.interact();
        for _ in 0..181 {
            g.simulate(0., 0., false, false, false);
        }
        assert!(g.breached);
        assert!(!g.blocked(world::prison::WEAK, 0.45));
        g.yaw = -std::f32::consts::FRAC_PI_2;
        for _ in 0..130 {
            g.simulate(1., 0., false, false, false);
        }
        assert!(g.player.pos.x < -175.);
        assert_eq!(g.player.team, Team::Criminal);
    }
    #[test]
    fn no_free_pistol_purchase_costs_exactly_350() {
        let mut g = Game::from_save(Default::default()).unwrap();
        assert!(!g.save.pistol_owned);
        assert_eq!(g.pistol.ammo, 0);
        g.slot = 1;
        g.fire();
        assert_eq!(g.pistol.ammo, 0);
        g.save.balance = 349;
        assert!(!g.buy_pistol());
        assert_eq!(g.save.balance, 349);
        g.save.balance = 550;
        assert!(g.buy_pistol());
        assert_eq!(g.save.balance, 200);
        assert_eq!(g.pistol.ammo, 12);
        assert!(!g.buy_pistol());
        assert_eq!(g.save.balance, 200);
    }
    #[test]
    fn stealing_any_car_changes_role_and_no_garage_purchase_is_needed() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        assert!(g.save.owned.is_empty());
        g.player.pos = g.cars[0].pos;
        g.player.pos.y = 1.;
        g.interact();
        assert_eq!(g.driving, Some(0));
        assert_eq!(g.player.team, Team::Criminal);
        assert_eq!(g.wanted, 1);
    }
    #[test]
    fn akari_spawns_can_be_stolen_and_uses_its_own_saved_tuning() {
        let mut save = save_system::Save::default();
        save.car_engine[7] = 2;
        save.car_brakes[7] = 1;
        save.car_paint[7] = [0.7, 0.15, 0.08];
        let mut g = Game::from_save(save).unwrap();
        let id = g.cars.iter().position(|c| c.kind == 7).unwrap();
        assert_eq!(g.cars[id].engine_level, 2);
        assert_eq!(g.cars[id].brake_level, 1);
        assert_eq!(g.cars[id].color, [0.7, 0.15, 0.08]);
        g.menu = false;
        g.player.pos = g.cars[id].pos;
        g.interact();
        assert_eq!(g.driving, Some(id));
        assert_eq!(g.player.team, Team::Criminal);
        assert_eq!(g.wanted, 1);
        let c = &mut g.cars[id];
        let start = c.pos;
        for _ in 0..120 {
            c.pos = vehicle_physics::step(c, 1., 0., false, STEP);
        }
        assert!(c.pos.distance(start) > 5.);
        assert!(c.speed > 0. && c.speed <= CATALOG[7].max_speed * 1.16);
        assert_eq!(g.save.car_engine[0], 0);
    }

    #[test]
    fn store_escape_pays_and_police_deadline_confiscates_loot() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs.clear();
        g.save.carry_level = 2;
        break_case(&mut g, 1);
        assert_eq!(g.alarm.site, Some(1));
        for _ in 0..200 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(g.alarm.loot, 80);
        assert_eq!(g.save.balance, 0);
        g.player.pos = Vec3::new(110., 1., 185.);
        g.simulate(0., 0., false, false, false);
        assert_eq!(g.save.balance, 80);
        assert!(g.alarm.site.is_none());
        break_case(&mut g, 2);
        for _ in 0..1801 {
            g.simulate(0., 0., false, false, false);
        }
        assert!(g.npcs.iter().any(|n| n.role == Role::Swat));
        // Reinforcements must approach and stun, never just deduct cash on timeout.
        assert!(g.alarm.dispatched);
        assert_eq!(g.save.balance, 80);
        let victim = g.player.pos;
        let mut officer = Npc::new(Role::Police, victim.x, victim.z - 1.5);
        officer.waypoint = victim;
        g.npcs = vec![officer];
        g.simulate(0., 0., false, false, false);
        assert!(g.cuffs.active);
        for _ in 0..301 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(g.player.team, Team::Prisoner);
        assert_eq!(g.alarm.loot, 0);
        assert_eq!(g.save.balance, 80);
    }
    #[test]
    fn dispatched_police_walk_to_store_and_arrest_staying_player() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs.clear();
        break_case(&mut g, 1);
        for _ in 0..4200 {
            g.simulate(0., 0., false, false, false);

            if g.player.team == Team::Prisoner {
                break;
            }
        }
        assert_eq!(
            g.player.team,
            Team::Prisoner,
            "police failed to reach player; positions {:?}; nearest {}",
            g.npcs
                .iter()
                .map(|n| (n.pos, n.waypoint))
                .collect::<Vec<_>>(),
            g.npcs
                .iter()
                .map(|n| n.pos.distance(SITES[1].center))
                .fold(f32::MAX, f32::min)
        );
        assert_eq!(g.alarm.loot, 0);
        assert_eq!(g.save.balance, 0);
    }
    #[test]
    fn stealing_response_car_cancels_scripted_movement() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.prepare_response(1);
        let id = g.response.as_ref().unwrap().1;
        g.steal(id);
        assert!(g.response.is_none());
        assert_eq!(g.driving, Some(id));
    }
    #[test]
    fn officers_cannot_tase_through_closed_door_or_other_floor() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.player.team = Team::Criminal;
        g.wanted = 3;
        g.player.pos = Vec3::new(320., 1., 17.);
        g.npcs = vec![Npc::new(Role::Police, 320., 12.)];
        g.tick_npcs();
        assert_eq!(g.stun, 0.);
        g.player.pos = Vec3::new(-116., -3., -140.);
        g.npcs = vec![Npc::new(Role::Police, -116., -120.)];
        g.tick_npcs();
        assert_eq!(g.stun, 0.);
    }
    #[test]
    fn npc_robber_is_not_arrested_through_closed_door() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs = vec![
            Npc::new(Role::Police, 320., 14.3),
            Npc::new(Role::Robber, 320., 16.),
        ];
        g.player.pos = Vec3::new(250., 1., 0.);
        g.tick_npcs();
        assert_eq!(g.npcs[1].disabled, 0.);
    }
    #[test]
    fn all_requested_npc_roles_exist() {
        let ns = crate::npcs::population();
        for role in [
            Role::Resident,
            Role::Robber,
            Role::Prisoner,
            Role::Police,
            Role::Swat,
        ] {
            assert!(ns.iter().any(|n| n.role == role));
        }
    }
    #[test]
    fn casino_stairs_support_player() {
        let mut g = Game::from_save(save_system::Save::default()).unwrap();
        g.menu = false;
        g.player = Player::new(Team::Criminal);
        g.player.pos = Vec3::new(377., 1., 227.);
        g.yaw = std::f32::consts::PI;
        for _ in 0..220 {
            g.simulate(1., 0., false, false, false);
        }
        assert!(g.player.pos.z > 248.1);
        assert!((g.player.pos.y - 5.).abs() < 0.05);
        assert!(!g.world.blocked(g.player.pos, 0.45));
    }
    #[test]
    fn sedan_traverses_bridge() {
        let world = World::new();
        let mut car = Car::new(0, Vec3::new(-495., 0.65, 0.));
        car.angle = std::f32::consts::FRAC_PI_2;
        for _ in 0..360 {
            let mut next = vehicle_physics::step(&mut car, 1., 0., false, STEP);
            next.y = world.surface(next.x, next.z) + 0.65;
            assert!(!world.blocked(next, 1.7));
            assert!((next.y - car.pos.y).abs() < 0.6);
            car.pos = next;
        }
        assert!(car.pos.x > -390.);
    }
    #[test]
    fn bank_timeout_discards_loot() {
        let mut b = robberies::Bank::default();
        b.start();
        b.tick(Vec3::default(), 71.);
        assert!(!b.active);
        assert_eq!(b.deliver(), 0);
    }
    #[test]
    fn interaction_spam_cannot_rob_and_impacts_are_cooldown_limited() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs.clear();
        let q = Alarm::case(2);
        g.player.pos = Vec3::new(q.x, 1., q.z - 2.);
        g.yaw = std::f32::consts::PI;
        for _ in 0..10 {
            g.interact();
        }
        assert!(g.alarm.site.is_none());
        for _ in 0..10 {
            g.fire();
        }
        for _ in 0..36 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(g.alarm.hits[2], 1);
        assert_eq!(g.alarm.loot, 0);
        for _ in 0..260 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(g.alarm.loot, 0);
    }
    #[test]
    fn collection_requires_holding_and_bank_capacity_upgrade() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs.clear();
        break_case(&mut g, 0);
        g.holding = false;
        for _ in 0..400 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(g.alarm.loot, 0);
        g.holding = true;
        for _ in 0..361 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(g.alarm.loot, 280);
        g.alarm.cancel();
        g.alarm.cooldown[0] = 0.;
        g.save.carry_level = 2;
        break_case(&mut g, 0);
        for _ in 0..361 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(g.alarm.loot, 420);
    }
    #[test]
    fn gym_charges_increasing_cost_and_caps_levels() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.save.balance = 1000;
        g.train(1);
        assert_eq!(g.save.run_level, 1);
        assert_eq!(g.save.balance, 940);
        g.train(2);
        assert_eq!(g.save.endurance_level, 0);
        g.training_time = 0.;
        g.train(2);
        assert_eq!(g.save.endurance_level, 1);
        assert_eq!(g.save.balance, 880);
        g.training_time = 0.;
        g.save.carry_level = 5;
        g.train(3);
        assert_eq!(g.save.balance, 880);
    }
    #[test]
    fn workshop_requires_location_and_charges_real_car_changes() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.open_workshop();
        assert!(g.workshop.is_none());
        g.player.pos = Vec3::new(430., 1., -70.);
        g.cars[0].pos = g.player.pos;
        g.cars[0].damage = 80.;
        g.driving = Some(0);
        g.save.balance = 64;
        g.open_workshop();
        assert_eq!(g.workshop, Some(0));
        g.service(1);
        assert_eq!(g.cars[0].damage, 0.);
        assert_eq!(g.save.balance, 24);
        let color = g.cars[0].color;
        g.service(2);
        assert_eq!(g.cars[0].color, color);
        g.save.balance = 25;
        g.service(2);
        assert_ne!(g.cars[0].color, color);
        assert_eq!(g.save.balance, 0);
    }
    #[test]
    fn supplies_require_vendor_and_are_consumed_from_hotbar() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.save.balance = 20;
        g.supplies(true);
        assert_eq!(g.save.water, 0);
        g.player.pos = Vec3::new(210., 1., 244.);
        g.supplies(true);
        assert_eq!(g.save.water, 1);
        assert_eq!(g.save.balance, 12);
        g.supplies(false);
        assert_eq!(g.save.food, 1);
        assert_eq!(g.save.balance, 0);
        g.slot = 3;
        g.stamina = 0.;
        g.fire();
        assert_eq!(g.save.water, 0);
        assert_eq!(g.stamina, 100.);
        g.slot = 4;
        g.player.health = 60.;
        g.fire();
        assert_eq!(g.player.health, 85.);
        assert_eq!(g.save.food, 0);
    }
    #[test]
    fn sprint_drains_stamina_and_idle_recovers_it() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs.clear();
        g.player.pos = Vec3::new(30., 1., -60.);
        for _ in 0..180 {
            g.simulate(1., 0., true, false, false);
        }
        assert!(g.stamina < 40.);
        for _ in 0..180 {
            g.simulate(0., 0., false, false, false);
        }
        assert!(g.stamina > 70.);
    }
    #[test]
    fn physical_sdl_keys_dispatch_cyrillic_and_modal_navigation() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.key_action(13);
        g.player.pos = world::prison::HATCH;
        let key = |scan| renderer_gl21::Event::synthetic_key(scan, 0x444).key();
        g.key_action(key(8));
        assert_eq!(g.player.pos.y, -3.);
        g.key_action(key(9));
        assert!(g.punch_time > 0.);
        g.key_action(key(16));
        assert!(g.map);
        g.key_action(key(10));
        assert!(g.garage && !g.map);
        g.key_action(key(10));
        assert!(!g.modal());
        g.player.pos = Vec3::new(430., 1., -70.);
        g.cars[0].pos = g.player.pos;
        g.key_action(key(13));
        assert_eq!(g.workshop, Some(0));
        g.key_action(27);
        g.player.pos = Vec3::new(60., 1., 240.);
        g.key_action(key(8));
        assert!(g.armory);
        g.key_action(key(10));
        assert!(g.garage && !g.armory);
    }
    #[test]
    fn armory_click_selection_purchase_equipment_and_ammo_are_real() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.armory = true;
        g.save.balance = 650;
        g.ui_click(430., 240.);
        assert_eq!(g.weapon_selection, 1);
        g.ui_click(250., 440.);
        assert!(g.save.weapon_owned[1]);
        assert_eq!(g.save.balance, 50);
        assert_eq!(g.pistol.kind, 1);
        g.pistol.ammo = 3;
        g.key_action(27);
        g.key_action(50);
        g.key_action(54);
        assert_eq!(g.pistol.ammo, 3);
        g.fire();
        assert_eq!(g.pistol.ammo, 2);
        g.armory = true;
        g.pistol.reserve = 0;
        g.ui_click(650., 440.);
        assert_eq!(g.save.balance, 0);
        assert!(g.pistol.reserve >= 18);
    }
    #[test]
    fn engine_brake_and_suspension_upgrades_charge_and_cap() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.workshop = Some(0);
        g.save.balance = 2000;
        g.service(3);
        g.service(4);
        g.service(5);
        assert_eq!(g.cars[0].engine_level, 1);
        assert_eq!(g.cars[0].brake_level, 1);
        assert_eq!(g.cars[0].suspension_level, 1);
        assert_eq!(g.save.balance, 1630);
        g.cars[0].engine_level = 5;
        g.service(3);
        assert_eq!(g.save.balance, 1630);
    }
    #[test]
    fn updated_tower_lift_reaches_furnished_floors_without_obstructions() {
        let mut g = Game::from_save(save_system::Save::default()).unwrap();
        g.menu = false;
        let b = g
            .world
            .buildings
            .iter()
            .find(|b| b.name == "Office" && b.p.x == 110. && b.p.z == 100.)
            .unwrap()
            .clone();
        g.player.pos = Vec3::new(b.p.x, b.p.y + 1., b.p.z - 8.);
        for floor in 1..=3 {
            assert!(!g.world.blocked(g.player.pos, 0.45));
            g.interact();
            assert_eq!(g.player.pos.y, b.p.y + floor as f32 * 4. + 1.);
            assert_eq!(b.rendered_floor(g.player.pos), floor);
            assert!(!g.world.blocked(g.player.pos, 0.45));
            assert!(b.interior_mesh_at(floor).vertices.len() < 5000);
        }
    }

    #[test]
    fn rent_expires_and_own_apartment_cannot_be_robbed() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs.clear();
        g.save.balance = 150;
        g.lease(0);
        assert_eq!(g.save.balance, 0);
        assert_eq!(g.save.leases[0], 900.);
        let q = Alarm::case(5);
        g.player.pos = Vec3::new(q.x, 1., q.z - 2.);
        g.yaw = std::f32::consts::PI;
        g.punch();
        for _ in 0..36 {
            g.simulate(0., 0., false, false, false);
        }
        assert!(g.alarm.site.is_none());
        assert!(g.save.leases[0] < 900.);
        g.save.leases[0] = 0.;
        break_case(&mut g, 5);
        for _ in 0..301 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(g.alarm.loot, 90);
    }
    #[test]
    fn factory_collects_capacity_limited_loot_and_penthouse_lift_works() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs.clear();
        break_case(&mut g, 3);
        for _ in 0..481 {
            g.simulate(0., 0., false, false, false);
        }
        assert_eq!(
            g.alarm.hits[3],
            3,
            "pos {:?} clear {} interior {:?}",
            g.player.pos,
            g.clear(g.player.pos, Alarm::case(3)),
            g.world.interior(g.player.pos)
        );
        assert_eq!(
            g.alarm.loot, 173,
            "progress {} y {}",
            g.alarm.progress, g.player.pos.y
        );
        g.alarm.cancel();
        g.player.pos = Vec3::new(350., 1., -65.);
        g.interact();
        assert_eq!(g.player.pos.y, 19.);
        g.interact();
        assert_eq!(g.player.pos.y, 1.);
    }
    #[test]
    fn crouch_and_crawl_keys_have_distinct_states_and_shorter_collision() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.key_action(99);
        assert!(g.crouched);
        g.key_action(122);
        assert!(g.prone && !g.crouched);
        let w = World::new();
        let p = Vec3::new(218., 1., 232.16);
        assert!(w.blocked(p, 0.3));
        assert!(!w.body_blocked(p, 0.3, 0.65));
    }
    #[test]
    fn paid_vehicle_plans_survive_restart() {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.workshop = Some(0);
        g.save.balance = 200;
        g.service(3);
        assert_eq!(g.save.car_engine[0], 1);
        let resumed = Game::from_save(g.save).unwrap();
        assert_eq!(resumed.cars[0].engine_level, 1);
    }
    #[test]
    fn dispatched_police_enter_factory_and_apartment() {
        for id in [3, 5] {
            let mut g = Game::from_save(Default::default()).unwrap();
            g.menu = false;
            g.npcs.clear();
            break_case(&mut g, id);
            for _ in 0..5400 {
                g.simulate(0., 0., false, false, false);
                if g.player.team == Team::Prisoner {
                    break;
                }
            }
            assert_eq!(
                g.player.team,
                Team::Prisoner,
                "site {} positions {:?}",
                id,
                g.npcs.iter().map(|n| n.pos).collect::<Vec<_>>()
            );
        }
    }
}

impl Game {
    fn draw_escape(&self, r: &mut Renderer) {
        if self.immunity > 0. {
            r.panel([330., 150., 370., 38.], [0.05, 0.5, 0.8], 0.9, 14.);
            text(
                r,
                345.,
                160.,
                2.,
                &format!("ARREST / STUN IMMUNITY {:.1}s", self.immunity),
                WHITE,
            );
        }
        if self.cuffs.active {
            r.panel([250., 200., 520., 160.], [0.04, 0.055, 0.08], 0.96, 20.);
            text(
                r,
                275.,
                220.,
                2.6,
                &format!(
                    "ESCAPE CUFFS {:.1}s  {}/{}",
                    self.cuffs.remaining, self.cuffs.success, self.cuffs.required
                ),
                GOLD,
            );
            r.panel([275., 275., 470., 28.], [0.15, 0.18, 0.23], 1., 8.);
            r.panel(
                [
                    275. + self.cuffs.target * 470.,
                    275.,
                    self.cuffs.width() * 470.,
                    28.,
                ],
                [0.1, 0.85, 0.45],
                1.,
                4.,
            );
            r.panel(
                [275. + self.cuffs.marker * 470., 271., 4., 36.],
                WHITE,
                1.,
                1.,
            );
            text(r, 275., 323., 2., "SPACE IN GREEN AREA", WHITE);
        }
    }
}
#[cfg(test)]
mod residential_regressions {
    use super::*;
    #[test]
    fn furniture_cash_is_paid_once_across_reload() {
        let mut g = Game::from_save(Default::default()).unwrap();
        let b = g
            .world
            .buildings
            .iter()
            .find(|b| b.name == "Office")
            .unwrap();
        let variant = (b.id * 13 + 7) % 32;
        let item = variant % 5;
        let (dx, dz) = [(0., 5.), (0., -5.), (0., 0.), (2.4, 6.), (-4., -5.)][item];
        let p = Vec3::new(
            b.p.x - 8. + dx,
            b.p.y + 5.,
            b.p.z + (variant % 4) as f32 - 1.5 + dz,
        );
        g.player.pos = p;
        let balance = g.save.balance;
        assert!(g.special_interact());
        assert_eq!(g.save.balance, balance + 35);
        let mut resumed = Game::from_save(g.save).unwrap();
        resumed.player.pos = p;
        assert!(resumed.special_interact());
        assert_eq!(resumed.save.balance, balance + 35);
    }
    #[test]
    fn low_wanted_does_not_track_through_building() {
        let mut g = Game::from_save(Default::default()).unwrap();
        let b = g
            .world
            .buildings
            .iter()
            .find(|b| b.name == "Office")
            .unwrap();
        g.player.pos = Vec3::new(b.p.x, b.p.y + 5., b.p.z);
        g.player.team = Team::Criminal;
        g.wanted = 1;
        let mut cop = crate::npcs::Npc::new(Role::Police, b.p.x, b.p.z - b.size.z / 2. - 4.);
        cop.pos.y = b.p.y + 1.;
        cop.home = cop.pos;
        g.npcs = vec![cop];
        for _ in 0..60 {
            g.tick_npcs();
        }
        assert!(!g.cuffs.active);
        assert_eq!(g.stun, 0.);
        assert!(g.npcs[0].waypoint.distance(g.player.pos) > 4.);
    }
}
