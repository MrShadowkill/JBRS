const BLUE: [f32; 3] = [0.035, 0.52, 0.88];
const ORANGE: [f32; 3] = [0.96, 0.46, 0.045];
const INK: [f32; 3] = [0.045, 0.055, 0.075];
impl Game {
    fn draw_interface(&self, r: &mut Renderer) {
        r.panel([18., 14., 160., 58.], INK, 0.88, 24.);
        text(r, 34., 27., 2.6, "JBRS", WHITE);
        r.panel([192., 14., 170., 58.], INK, 0.88, 22.);
        text(
            r,
            209.,
            26.,
            3.7,
            &format!("${}", self.save.balance),
            [0.4, 0.9, 0.28],
        );
        for (x, label) in if crate::mobile::enabled() {
            [(378., "MAP"), (482., "SET"), (586., "GAR")]
        } else {
            [(378., "MAP  M"), (482., "SET  G"), (586., "GAR  J")]
        } {
            gui::button(r, [x, 14., 94., 58.], label, INK);
        }
        r.panel([18., 83., 245., 58.], INK, 0.83, 18.);
        text(
            r,
            32.,
            94.,
            1.6,
            &format!("{:?}  WANTED {}", self.player.team, self.wanted),
            ORANGE,
        );
        text(
            r,
            32.,
            115.,
            1.5,
            &format!("HP {:.0}  STAMINA {:.0}", self.player.health, self.stamina),
            WHITE,
        );
        r.panel([820., 14., 186., 211.], INK, 0.86, 22.);
        self.draw_map(r, [834., 28., 158., 158.]);
        text(r, 850., 196., 1.7, "M OPEN MAP", WHITE);
        if !self.modal() {
            for (i, label) in ["HANDS", "PISTOL", "C4", "WATER", "FOOD", "SHOTGUN", "RIFLE"]
                .iter()
                .enumerate()
            {
                let x = 222. + i as f32 * 82.;
                r.panel(
                    [x, 496., 76., 62.],
                    if i == self.slot { BLUE } else { INK },
                    0.92,
                    16.,
                );
                text(r, x + 8., 504., 1.7, &format!("{}", i + 1), WHITE);
                let locked = match i {
                    1 => !self.save.weapon_owned[0],
                    5 => !self.save.weapon_owned[1],
                    6 => !self.save.weapon_owned[2],
                    _ => false,
                };
                text(
                    r,
                    x + 7.,
                    535.,
                    1.2,
                    if locked { "LOCKED" } else { label },
                    WHITE,
                );
                if matches!(i, 1 | 5 | 6) {
                    weapon_icon_small(
                        r,
                        x + 25.,
                        514.,
                        if i == 1 {
                            0
                        } else if i == 5 {
                            1
                        } else {
                            2
                        },
                        WHITE,
                    );
                } else {
                    r.panel([x + 30., 518., 16., 12.], ORANGE, 1., 4.);
                }
            }
            text(
                r,
                22.,
                550.,
                1.2,
                if crate::mobile::enabled() {
                    "MOVE: LEFT   LOOK: RIGHT"
                } else {
                    "TAB CURSOR   C CROUCH   Z CRAWL"
                },
                WHITE,
            );
            if let Some(id) = self.world.interior(self.player.pos) {
                r.panel([335., 425., 355., 40.], INK, 0.84, 18.);
                text(
                    r,
                    351.,
                    436.,
                    1.7,
                    &format!(
                        "{}   [E]",
                        if self.world.buildings[id].name == "Office" {
                            "APARTMENTS"
                        } else {
                            self.world.buildings[id].name
                        }
                    ),
                    WHITE,
                );
            }
            if matches!(self.slot, 1 | 5 | 6) {
                text(
                    r,
                    380.,
                    470.,
                    1.6,
                    &format!(
                        "{}  {}/{}  R RELOAD",
                        weapons::CATALOG[self.pistol.kind].name,
                        self.pistol.ammo,
                        self.pistol.reserve
                    ),
                    WHITE,
                );
            }
            r.rect(507., 285., 10., 2., WHITE);
            r.rect(511., 281., 2., 10., WHITE);
        }
        if self.message_time > 0. {
            r.panel([270., 83., 535., 35.], INK, 0.85, 16.);
            text(r, 284., 92., 1.25, &self.message, WHITE);
        }
        if let Some(id) = self.alarm.site {
            r.panel([18., 156., 245., 92.], INK, 0.92, 20.);
            text(r, 33., 170., 2., SITES[id].name, ORANGE);
            text(
                r,
                33.,
                199.,
                1.7,
                &format!("POLICE IN {:.1}S", self.alarm.remaining),
                WHITE,
            );
            let weight: f32 = if id == 0 || id == 3 || id == 4 {
                12.
            } else if id >= 5 {
                6.
            } else {
                4.
            };
            let cap = (8 + self.save.carry_level * 2) as f32;
            let fill = (self.alarm.progress / SITES[id].collect).min(1.) * weight.min(cap);
            r.panel([820., 250., 186., 105.], INK, 0.9, 20.);
            text(r, 835., 264., 2.2, "LOOT BAG", WHITE);
            text(
                r,
                835.,
                293.,
                1.6,
                &format!("{:.1}/{:.0} KG  ${}", fill, cap, self.alarm.loot),
                ORANGE,
            );
            r.panel([835., 326., 156., 10.], [0.16, 0.19, 0.23], 1., 5.);
            if fill > 0. {
                r.panel([835., 326., 156. * fill / cap, 10.], BLUE, 1., 5.);
            }
            text(
                r,
                290.,
                387.,
                1.7,
                &format!(
                    "LOCK {}/3  HOLD E {:.1}/{:.0}S",
                    self.alarm.hits[id], self.alarm.progress, SITES[id].collect
                ),
                WHITE,
            );
        }
        if let Some(car) = self.driving {
            r.panel([18., 435., 186., 96.], INK, 0.9, 22.);
            text(
                r,
                34.,
                451.,
                3.5,
                &format!("{:.0} KM/H", self.cars[car].speed.abs() * 3.6),
                WHITE,
            );
            text(r, 34., 494., 1.4, CATALOG[self.cars[car].kind].name, ORANGE);
        }
        if self.stun > 0. {
            text(r, 415., 225., 4., "STUNNED", BLUE);
        }
        if let Some(t) = self.fuse {
            text(r, 445., 260., 3., &format!("C4 {t:.1}"), ORANGE);
        }
        if self.training_time > 0. {
            text(r, 410., 230., 3., "TRAINING", ORANGE);
        }
        if self.menu {
            r.panel([174., 125., 676., 363.], INK, 0.92, 28.);
            text(r, 328., 150., 5.2, "JBRS", WHITE);
            text(r, 368., 202., 1.5, "CHOOSE YOUR SIDE", [0.65, 0.7, 0.78]);
            gui::button(r, [230., 242., 260., 127.], "PRISONER [1]", ORANGE);
            text(r, 267., 329., 1.6, "PLAN YOUR ESCAPE", WHITE);
            gui::button(
                r,
                [534., 242., 260., 127.],
                "CRIMINAL [2]",
                [0.75, 0.2, 0.15],
            );
            text(r, 601., 329., 1.6, "TAKE THE RISK", WHITE);
            gui::button(
                r,
                [280., 405., 300., 54.],
                "PLAY / RESUME  ENTER",
                [0.12, 0.7, 0.4],
            );
            gui::button(r, [605., 405., 180., 54.], "SETTINGS  G", INK);
        }
        if self.garage {
            modal_title(r, "GRAPHICS SETTINGS");
            let names = [
                format!(
                    "QUALITY   {}",
                    crate::quality::PRESETS[self.save.quality].name
                ),
                format!(
                    "TEXTURES   {} X {}",
                    [64, 128, 256, 512][self.save.texture_level],
                    [64, 128, 256, 512][self.save.texture_level]
                ),
                format!(
                    "OBJECT SHADOWS   {}",
                    if self.save.object_shadows {
                        "ON"
                    } else {
                        "OFF"
                    }
                ),
            ];
            for (i, n) in names.iter().enumerate() {
                gui::button(
                    r,
                    [206., 205. + i as f32 * 70., 612., 54.],
                    n,
                    if self.graphics_row == i {
                        BLUE
                    } else {
                        [0.12, 0.15, 0.2]
                    },
                );
            }
            text(
                r,
                230.,
                442.,
                1.8,
                "UP/DOWN SELECT   LEFT/RIGHT CHANGE   ESC CLOSE",
                WHITE,
            );
        }
        if let Some(id) = self.workshop {
            if self.workshop_config {
                self.draw_customs(r);
                return;
            }
            modal_title(r, "CUSTOMS GARAGE");
            let car = &self.cars[id];
            text(r, 214., 199., 2.2, CATALOG[car.kind].name, WHITE);
            text(
                r,
                535.,
                199.,
                1.8,
                &format!("DAMAGE {:.0}%", car.damage),
                ORANGE,
            );
            let labels = [
                "1  REPAIR  $40".to_string(),
                "2  REPAINT  $25".to_string(),
                format!(
                    "3  ENGINE LV{}  ${}",
                    car.engine_level,
                    150 + car.engine_level * 100
                ),
                format!(
                    "4  BRAKES LV{}  ${}",
                    car.brake_level,
                    100 + car.brake_level * 80
                ),
                format!(
                    "5  SUSPENSION LV{}  ${}",
                    car.suspension_level,
                    120 + car.suspension_level * 100
                ),
            ];
            for (i, n) in labels.iter().enumerate() {
                gui::button(
                    r,
                    [
                        210. + (i % 2) as f32 * 310.,
                        244. + (i / 2) as f32 * 72.,
                        288.,
                        55.,
                    ],
                    n,
                    if i >= 2 { BLUE } else { [0.16, 0.2, 0.27] },
                );
            }
            text(
                r,
                530.,
                409.,
                1.5,
                &format!(
                    "TOP SPEED {:.0} KM/H",
                    CATALOG[car.kind].max_speed * (1. + car.engine_level as f32 * 0.08) * 3.6
                ),
                WHITE,
            );
        }
        if self.workshop.is_some() {
            gui::button(
                r,
                [530., 444., 280., 28.],
                "N  SELECT CAR / ENGINE / COLORS",
                ORANGE,
            );
        }
        if self.armory {
            modal_title(r, "WEAPON SHOP");
            for (i, spec) in weapons::CATALOG.iter().enumerate() {
                let x = 205. + i as f32 * 207.;
                r.panel(
                    [x, 210., 190., 190.],
                    if i == self.weapon_selection {
                        BLUE
                    } else {
                        [0.13, 0.16, 0.22]
                    },
                    0.96,
                    20.,
                );
                text(r, x + 17., 228., 2.5, spec.name, WHITE);
                weapon_icon(r, x + 25., 279., i, WHITE);
                text(
                    r,
                    x + 17.,
                    337.,
                    2.4,
                    &format!("${}", spec.price),
                    [0.55, 1., 0.35],
                );
                text(
                    r,
                    x + 17.,
                    371.,
                    1.4,
                    &format!("DAMAGE {:.0}  MAG {}", spec.damage, spec.magazine),
                    WHITE,
                );
            }
            gui::button(
                r,
                [206., 426., 380., 52.],
                if self.save.weapon_owned[self.weapon_selection] {
                    "EQUIP  ENTER"
                } else {
                    "BUY SELECTED  ENTER"
                },
                ORANGE,
            );
            gui::button(r, [610., 426., 209., 52.], "AMMO  $50", BLUE);
        }
        if let Some(id) = self.housing {
            modal_title(r, "PROPERTY / RENTAL");
            text(
                r,
                218.,
                210.,
                2.6,
                ["APARTMENT A", "APARTMENT B", "PENTHOUSE", "SAFEHOUSE"][id],
                WHITE,
            );
            text(
                r,
                218.,
                255.,
                2.,
                &format!(
                    "LEASE {:.0} MIN  PRICE ${}",
                    self.save.leases[id] / 60.,
                    [150, 180, 500, 100][id]
                ),
                ORANGE,
            );
            gui::button(r, [211., 310., 286., 60.], "1 RENT  15 MIN", BLUE);
            gui::button(
                r,
                [530., 310., 286., 60.],
                if id < 2 { "2 BREAK IN" } else { "2 ENTER HOME" },
                ORANGE,
            );
            gui::button(
                r,
                [211., 405., 605., 60.],
                "3 REST AT YOUR RENTED HOME",
                [0.15, 0.2, 0.25],
            );
        }
        if self.training {
            modal_title(r, "UNDERGROUND GYM");
            for (i, (n, l)) in [
                ("RUN", self.save.run_level),
                ("ENDURANCE", self.save.endurance_level),
                ("CARRY", self.save.carry_level),
            ]
            .iter()
            .enumerate()
            {
                gui::button(
                    r,
                    [209., 214. + i as f32 * 76., 610., 56.],
                    &format!("{} {}   LEVEL {}/5   ${}", i + 1, n, l, 60 + l * 40),
                    BLUE,
                );
            }
        }
        if self.map {
            modal_title(r, "WORLD MAP / CLICK A DESTINATION");
            self.draw_map(r, [205., 195., 610., 280.]);
        }
    }
    fn ui_click(&mut self, x: f32, y: f32) {
        if self.customs_click(x, y) {
            return;
        }

        let hit = |rect| gui::contains(rect, x, y);
        if hit([378., 14., 94., 58.]) || hit([820., 14., 186., 211.]) {
            self.key_action(109);
            return;
        }
        if hit([482., 14., 94., 58.]) {
            self.key_action(103);
            return;
        }
        if hit([586., 14., 94., 58.]) {
            self.key_action(106);
            return;
        }
        if self.modal() && hit([794., 142., 40., 36.]) {
            self.close_panels();
            return;
        }
        if self.menu {
            for (rect, key) in [
                ([230., 242., 260., 127.], 49),
                ([534., 242., 260., 127.], 50),
                ([280., 405., 300., 54.], 13),
                ([605., 405., 180., 54.], 103),
            ] {
                if hit(rect) {
                    self.key_action(key);
                    return;
                }
            }
        } else if self.garage {
            for i in 0..3 {
                if hit([206., 205. + i as f32 * 70., 612., 54.]) {
                    self.graphics_row = i;
                    self.key_action(1073741903);
                    return;
                }
            }
        } else if self.workshop.is_some() {
            for i in 0..5 {
                if hit([
                    210. + (i % 2) as f32 * 310.,
                    244. + (i / 2) as f32 * 72.,
                    288.,
                    55.,
                ]) {
                    self.service(i as u32 + 1);
                    return;
                }
            }
        } else if self.armory {
            for i in 0..3 {
                if hit([205. + i as f32 * 207., 210., 190., 190.]) {
                    self.weapon_selection = i;
                    return;
                }
            }
            if hit([206., 426., 380., 52.]) {
                self.buy_weapon(self.weapon_selection);
            } else if hit([610., 426., 209., 52.]) {
                self.buy_ammo();
            }
        } else if self.housing.is_some() {
            for (rect, key) in [
                ([211., 310., 286., 60.], 49),
                ([530., 310., 286., 60.], 50),
                ([211., 405., 605., 60.], 51),
            ] {
                if hit(rect) {
                    self.key_action(key);
                    return;
                }
            }
        } else if self.training {
            for i in 0..3 {
                if hit([209., 214. + i as f32 * 76., 610., 56.]) {
                    self.train(i as u32 + 1);
                    return;
                }
            }
        } else if self.map && hit([205., 195., 610., 280.]) {
            self.waypoint = Some(Vec3::new(
                (x - 205.) / 610. * 2048. - 1024.,
                0.,
                (y - 195.) / 280. * 2048. - 1024.,
            ));
        } else if !self.modal() {
            for i in 0..7 {
                if hit([222. + i as f32 * 82., 496., 76., 62.]) {
                    self.key_action(49 + i);
                    return;
                }
            }
        }
    }
    fn buy_ammo(&mut self) {
        let id = self.weapon_selection;
        if !self.save.weapon_owned[id] || self.save.balance < 50 {
            self.notify("BUY THE WEAPON FIRST / NEED $50");
            return;
        }
        let current = if self.pistol.kind == id {
            self.pistol.reserve
        } else {
            self.save.ammunition[id]
        };
        if current >= weapons::CATALOG[id].reserve + self.save.carry_level * 24 {
            self.notify("AMMO BAG FULL - NO CHARGE");
            return;
        }
        self.save.balance -= 50;
        let count = weapons::CATALOG[id].reserve;
        self.save.ammunition[id] =
            (self.save.ammunition[id] + count).min(count + self.save.carry_level * 24);
        if self.pistol.kind == id {
            self.pistol.reserve =
                (self.pistol.reserve + count).min(count + self.save.carry_level * 24);
        }
        self.notify("AMMUNITION PURCHASED");
    }
}
fn modal_title(r: &mut Renderer, title: &str) {
    r.panel([174., 125., 676., 376.], INK, 0.95, 28.);
    text(r, 209., 148., 3.4, title, WHITE);
    gui::button(r, [794., 142., 40., 36.], "X", [0.22, 0.26, 0.32]);
}
fn weapon_icon(r: &mut Renderer, x: f32, y: f32, kind: usize, c: [f32; 3]) {
    weapon_icon_scaled(r, x, y, kind, c, 1.);
}
fn weapon_icon_small(r: &mut Renderer, x: f32, y: f32, kind: usize, c: [f32; 3]) {
    weapon_icon_scaled(r, x, y, kind, c, 0.38);
}
fn weapon_icon_scaled(r: &mut Renderer, x: f32, y: f32, kind: usize, c: [f32; 3], scale: f32) {
    let rect =
        |dx: f32, dy: f32, w: f32, h: f32| [x + dx * scale, y + dy * scale, w * scale, h * scale];
    let length = match kind {
        0 => 30.,
        1 => 90.,
        _ => 104.,
    };
    r.panel(rect(0., 0., length, 10.), c, 1., 3. * scale);
    r.panel(rect(6., 7., 9., 21.), c, 1., 2. * scale);
    if kind > 0 {
        r.panel(rect(35., 10., 12., 18.), c, 1., 2. * scale);
        r.panel(rect(-5., 7., 20., 12.), c, 1., 3. * scale);
    }
    if kind == 2 {
        r.panel(rect(28., -7., 32., 6.), BLUE, 1., 2. * scale);
    }
}
fn draw_weapon_model(r: &mut Renderer, p: Vec3, a: f32, kind: usize) {
    let local = |x: f32, y: f32, z: f32| {
        [
            p.x + x * a.cos() + z * a.sin(),
            p.y + y,
            p.z - x * a.sin() + z * a.cos(),
        ]
    };
    let length = match kind {
        0 => 0.4,
        1 => 0.95,
        _ => 1.05,
    };
    r.cube(
        local(0.5, -0.35, 0.15),
        [0.12, 0.15, length],
        [0.15, 0.18, 0.2],
        a,
    );
    r.cube(
        local(0.5, -0.49, -0.08),
        [0.11, 0.25, 0.12],
        [0.23, 0.19, 0.15],
        a,
    );
    r.cube(
        local(0.5, -0.26, -0.02),
        [0.08, 0.035, 0.1],
        [0.07, 0.09, 0.1],
        a,
    );
    if kind > 0 {
        r.cube(
            local(0.5, -0.44, -0.3),
            [0.17, 0.17, 0.3],
            [0.3, 0.21, 0.13],
            a,
        );
        r.cube(
            local(0.5, -0.41, 0.38),
            [0.16, 0.13, 0.28],
            [0.25, 0.22, 0.18],
            a,
        );
    }
    if kind == 2 {
        r.cube(
            local(0.5, -0.18, 0.08),
            [0.095, 0.09, 0.25],
            [0.09, 0.11, 0.13],
            a,
        );
        r.cube(
            local(0.5, -0.51, 0.1),
            [0.1, 0.22, 0.17],
            [0.13, 0.15, 0.17],
            a,
        );
    }
}
fn draw_crawler(r: &mut Renderer, p: Vec3, a: f32, c: [f32; 3], t: f32, motion: f32) {
    let local = |x: f32, y: f32, z: f32| {
        [
            p.x + x * a.cos() + z * a.sin(),
            p.y + y,
            p.z - x * a.sin() + z * a.cos(),
        ]
    };
    r.bone(
        local(0., -0.65, -0.3),
        local(0., -0.65, 0.35),
        0.74,
        0.32,
        a,
        c,
    );
    r.cube(
        local(0., -0.57, 0.55),
        [0.46, 0.43, 0.45],
        [0.76, 0.59, 0.42],
        a,
    );
    for side in [-1., 1.] {
        let swing = (t * 8.).sin() * side * motion * 0.18;
        r.bone(
            local(side * 0.22, -0.68, -0.25),
            local(side * 0.24, -0.84, -0.95 + swing),
            0.24,
            0.25,
            a,
            [0.16, 0.19, 0.25],
        );
        r.bone(
            local(side * 0.5, -0.63, 0.2),
            local(side * 0.55, -0.85, 0.75 - swing),
            0.23,
            0.23,
            a,
            c,
        );
    }
}
