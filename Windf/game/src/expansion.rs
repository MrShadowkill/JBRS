impl Game {
    fn special_interact(&mut self) -> bool {
        let p = self.player.pos;
        if p.distance(Vec3::new(-290., self.world.surface(-290., 100.) + 1., 100.)) < 4.
            && self.heists.bag > 0
        {
            if self
                .heists
                .site
                .is_some_and(|id| self.heists.solved < crate::heists::SITES[id].questions)
            {
                self.notify("COMPLETE SECURITY PUZZLES BEFORE DELIVERY");
                return true;
            }
            if self.heists.site==Some(0) && !self.heists.jewelry_roof_reached {
                self.notify("J-STORE: REACH ROOF WITH JEWELS BEFORE DELIVERY");
                return true;
            }
            let money = self.heists.bag;
            self.save.balance += money;
            self.heists.bag = 0;
            self.heists.finish();
            self.notify(&format!("COLLECTOR PAID ${money}"));
            return true;
        }
        if let Some(wagon) = self.heists.riding {
            if self.slot == 2
                && self.explosive
                && self.heists.gold(wagon)
                && self.heists.site == Some(5)
                && !self.heists.opened
                && self.heists.bomb <= 0.
            {
                self.explosive = false;
                self.heists.bomb = 3.;
                self.notify("C4 PLANTED - VAULT OPENS IN 3S");
                return true;
            }
            if self.heists.opened && self.heists.site == Some(5) && self.heists.bag == 0 {
                self.notify("HOLD E TO LOAD GOLD");
                return true;
            }
            self.heists.riding = None;
            let (q, a) = crate::heists::Heists::train(self.clock, wagon);
            self.player.pos = Vec3::new(
                q.x + a.cos() * 5.,
                self.world.surface(q.x + a.cos() * 5., q.z - a.sin() * 5.) + 1.,
                q.z - a.sin() * 5.,
            );
            self.notify("LEFT TRAIN");
            return true;
        }
        if self.driving.is_none() {
            if let Some(wagon) =
                (0..25).find(|i| crate::heists::Heists::train(self.clock, *i).0.distance(p) < 6.)
            {
                self.heists.riding = Some(wagon);
                if self.heists.gold(wagon) && self.heists.start(5) {
                    self.player.team = Team::Criminal;
                    self.wanted = self.wanted.max(3);
                    self.notify("GOLD WAGON - SELECT C4 THEN E");
                } else {
                    self.notify("BOARDED TRAIN - E TO LEAVE");
                }
                return true;
            }
        }
        if let Some(id) = self.world.interior(p) {
            let b = &self.world.buildings[id];
            let name = b.name;
            if matches!(name, "Office" | "Westhaven" | "Jewelry" | "Casino") {
                let lift = Vec3::new(
                    b.p.x + if name == "Casino" { 8. } else { 0. },
                    p.y,
                    b.p.z + if name == "Casino" { 9. } else { -8. },
                );
                if p.distance(lift) < 2.5 {
                    if name == "Jewelry" && self.heists.site == Some(0) && self.heists.strikes < 3 {
                        self.notify("BREAK DISPLAY GLASS FIRST - F");
                        return true;
                    }
                    if name=="Jewelry" && self.heists.site==Some(0) && self.heists.bag>0 {
                        self.notify("J-STORE SECURITY LOCK: USE SIDE RAMPS TO ROOF");
                        return true;
                    }
                    let max = if matches!(name, "Office" | "Westhaven") {
                        (b.size.y / 4.) as u32 - 1
                    } else if name == "Jewelry" {
                        3
                    } else {
                        2
                    };
                    let floor = ((p.y - b.p.y - 0.6) / 4.).floor().max(0.) as u32;
                    self.player.pos.y = b.p.y + ((floor + 1) % (max + 1)) as f32 * 4. + 1.;
                    self.player.vy = 0.;
                    self.camera = None;
                    self.notify("LIFT - E NEXT FLOOR / ROOF; P PARACHUTE");
                    return true;
                }
            }
            if matches!(name, "Office" | "Westhaven") {
                if let Some((key, stash, label)) = buildings::apartments::search_target(b, p) {
                    if self.save.searched_furniture.contains(&key) {
                        self.notify("ALREADY SEARCHED - EMPTY");
                    } else {
                        self.save.searched_furniture.push(key);
                        self.punch_time = 0.55;
                        self.punch_done = true;
                        if stash && self.player.team != Team::Police {
                            self.save.balance += 35;
                            self.player.team = Team::Criminal;
                            self.wanted = self.wanted.max(1);
                            self.bounty += 35;
                            self.notify(&format!("{label}: HIDDEN CASH +$35"));
                        } else { self.notify(&format!("{label}: NO STASH")); }
                    }
                    return true;
                }
                self.notify("LOBBY LIFT: E / UPSTAIRS: E NEAR FURNITURE TO SEARCH");
                return true;
            }
            if let Some(site) = crate::heists::SITES.iter().position(|s| s.name == name) {
                if self.player.team == Team::Police {
                    return false;
                }
                if self.heists.site.is_none() {
                    if self.alarm.site.is_some() {
                        self.notify("FINISH CURRENT ROBBERY FIRST");
                        return true;
                    }
                    if !self.heists.start(site) {
                        self.notify(&format!(
                            "CLOSED - REOPENS IN {:.0}s",
                            self.heists.cooldown[site]
                        ));
                        return true;
                    }
                    self.player.team = Team::Criminal;
                    self.wanted = self.wanted.max(3);
                    self.bounty += 150;
                }
                if self.heists.site != Some(site) {
                    return true;
                }
                if self.heists.solved < crate::heists::SITES[site].questions {
                    if site == 3 && p.y < b.p.y + 8.5 {
                        self.notify("USE LIFT TO SECURITY PC ON FLOOR 3");
                        return true;
                    }
                    if site != 1 || self.heists.bag > 0 {
                        self.heists.question();
                        return true;
                    }
                }
                self.notify(if matches!(site, 0 | 4) && self.heists.strikes < 3 {
                    if site==0 {"J-STORE: F BREAK DISPLAY; UPSTAIRS HOLD E TO LOAD JEWELS"} else {"F BREAK GLASS; HOLD E TO COLLECT"}
                } else {
                    "HOLD E AT LOOT TO FILL BAG; DELIVER TO COLLECTOR"
                });
                return true;
            }
        }
        false
    }
    fn special_strike(&mut self) -> bool {
        let Some(id) = self.heists.site.filter(|i| *i == 0 || *i == 4) else {
            return false;
        };
        let spec = crate::heists::SITES[id];
        let p = self.player.pos;
        if (p.y-self.world.surface(spec.x,spec.z)-1.).abs()<1.0 && [-1., 1.].iter().any(|side| {
            p.distance(Vec3::new(
                spec.x + side * if id == 0 { 6. } else { 8. },
                self.world.surface(spec.x, spec.z) + 1.,
                spec.z,
            )) < 2.6
        }) {
            self.heists.strikes = (self.heists.strikes + 1).min(3);
            self.notify(if self.heists.strikes >= 3 {
                if id == 4 {
                    "GLASS BROKEN - HOLD E THEN LEAVE AREA"
                } else {
                    "J-STORE GLASS BROKEN - UPSTAIRS FOR DIAMONDS"
                }
            } else {
                "GLASS CRACKED"
            });
        }
        true
    }
    fn tick_heists(&mut self) {
        self.heists.tick(STEP);
        if let Some(w) = self.heists.riding {
            let (p, _) = crate::heists::Heists::train(self.clock, w);
            self.player.pos = Vec3::new(p.x, p.y + 2.8, p.z);
            self.player.vy = 0.;
        }
        let Some(id) = self.heists.site else {
            return;
        };
        let spec = crate::heists::SITES[id];
        let deadline = if id == 3 {
            300. - self.wanted.min(5) as f32 * 40.
        } else {
            spec.delay / (1. + self.wanted.saturating_sub(1) as f32 * 0.12)
        };
        if self.heists.elapsed >= deadline && !self.heists.dispatched {
            self.heists.dispatched = true;
            let p = self.player.pos;
            let center = self
                .world
                .interior(p)
                .map(|i| self.world.buildings[i].entrance())
                .unwrap_or(p);
            for j in 0..6 {
                let q = Vec3::new(center.x - 6. + j as f32 * 2.4, center.y, center.z - 4.);
                let mut n = Npc::new(if j > 3 { Role::Swat } else { Role::Police }, q.x, q.z);
                n.pos.y = self.world.walking_surface(q) + 1.;
                n.home = n.pos;
                self.npcs.push(n);
            }
            self.notify("POLICE PERIMETER - EXITS BLOCKED");
        }
        let p = self.player.pos;
        let near = if id == 5 {
            self.heists.riding.is_some()
        } else {
            (p.x - spec.x).abs() < 12. && (p.z - spec.z).abs() < 10.
        };
        let floor = self
            .world
            .buildings
            .iter()
            .find(|b| b.name == spec.name)
            .map(|b| b.p.y)
            .unwrap_or(0.);
        if id==0 && self.heists.bag>0 && (p.x-spec.x).abs()<14. && (p.z-spec.z).abs()<14.
            && p.y>=floor+buildings::jewelry::ROOF+0.8 {
            self.heists.jewelry_roof_reached=true;
        }
        let ready = match id {
            0 => self.heists.strikes >= 3 && p.y > floor + 4.5 && p.y < floor + 7.5,
            1 => true,
            3 => self.heists.solved >= 5 && p.y < floor + 3.,
            4 => self.heists.strikes >= 3,
            5 => self.heists.opened,
            _ => self.heists.solved >= spec.questions,
        };
        if self.holding && near && ready && !self.heists.quiz && self.heists.bag == 0 {
            self.heists.progress += STEP;
            if self.heists.progress >= 8. {
                self.heists.bag = spec.reward * (8 + self.save.carry_level * 2).min(12) / 12;
                self.notify("BAG FULL - SOLVE EXIT IF NEEDED; DELIVER TO CRIMINAL BASE");
            }
        }
        if id == 4
            && self.heists.bag > 0
            && !self.heists.dispatched
            && p.distance(Vec3::new(spec.x, floor + 1., spec.z)) > 40.
        {
            let cash = self.heists.bag;
            self.save.balance += cash;
            self.heists.bag = 0;
            self.heists.finish();
            self.wanted = 0;
            self.notify(&format!("ESCAPED GAS STATION WITH ${cash}"));
        }
        // J-Store beams use the same positions for visible geometry and collision.
        if id==0 && !self.heists.quiz {
            let local=Vec3::new(p.x-spec.x,p.y-floor,p.z-spec.z);
            if buildings::jewelry::beams(self.clock).iter().any(|beam|beam.touches(local,self.stance)) {
                self.player.health=(self.player.health-24.*STEP).max(0.);
            }
        }
        if id==3 && near && !self.heists.quiz {
            let local=(p.z-spec.z+8.).rem_euclid(4.);
            if !(0.35..=3.65).contains(&local) && self.stance==0 && (p.y-floor-1.).abs()<0.25 {
                self.player.health=(self.player.health-20.*STEP).max(0.);
            }
        }
    }
    fn draw_heists(&self, r: &mut Renderer) {
        if self.heists.progress > 0.2 || self.heists.bag > 0 {
            let p = self.player.pos;
            let a = self.player.heading;
            r.rounded(
                [p.x - a.sin() * 0.4, p.y + 0.2, p.z - a.cos() * 0.4],
                [0.65, 0.9, 0.3],
                [0.3, 0.2, 0.12],
                a,
                8,
                0.4,
            );
        }
        let [jx,jz]=buildings::jewelry::CENTER;
        let floor = self.world.surface(buildings::jewelry::CENTER[0],buildings::jewelry::CENTER[1]);
        if self.heists.site==Some(0) && self.player.pos.distance(Vec3::new(buildings::jewelry::CENTER[0],floor+1.,buildings::jewelry::CENTER[1]))<80. {
            for beam in buildings::jewelry::beams(self.clock) {
                let p=[buildings::jewelry::CENTER[0]+beam.p[0],floor+beam.p[1],buildings::jewelry::CENTER[1]+beam.p[2]];
                r.cube(p,beam.size,[1.,0.055,0.035],0.);
                // Bright core supplies the stylized light strip without a bloom dependency.
                r.cube(p,beam.size.map(|v|(v*0.45).max(0.025)),[1.,0.65,0.35],0.);
            }
        }
        if self.player.pos.distance(Vec3::new(jx, floor + 1., jz)) < 60. {
            for side in [-1., 1.] {
                if self.heists.site != Some(0) || self.heists.strikes < 3 {
                    for edge in [-1.,1.] {
                        r.glass([jx+side*6.+edge*1.5,floor+1.64,jz],[0.025,0.6,5.],[0.25,0.7,0.82],0.);
                        r.glass([jx+side*6.,floor+1.64,jz+edge*2.5],[3.,0.6,0.025],[0.25,0.7,0.82],0.);
                    }
                    r.glass(
                        [jx + side * 6., floor + 1.95, jz],
                        [3., 0.04, 5.],
                        [0.15, 0.65, 0.85],
                        0.,
                    );
                } else {
                    for shard in 0..8 {
                        r.cube(
                            [
                                jx + side * 6. - 1. + shard as f32 * 0.3,
                                floor + 0.03,
                                jz - (shard % 3) as f32 * 0.5,
                            ],
                            [0.2, 0.04, 0.25],
                            [0.55, 0.8, 0.9],
                            shard as f32,
                        );
                    }
                }
            }
        }
        let quality=crate::quality::PRESETS[self.save.quality];
        let eye=self.camera.unwrap_or([self.player.pos.x,self.player.pos.y,self.player.pos.z]);
        let aspect=r.width as f32/r.height as f32;
        let visible=|p:Vec3| crate::world_view::sphere_visible([p.x,p.y+1.,p.z],7.,eye,self.yaw,self.pitch,aspect,quality.distance);
        for wagon in 0..25 {
            let (p, a) = crate::heists::Heists::train(self.clock, wagon);
            if p.distance(self.player.pos) > quality.distance || !visible(p) {
                continue;
            }
            if p.distance(self.player.pos)>quality.detail {
                r.cube([p.x,p.y,p.z],[3.5,1.4,8.5],[0.12,0.16,0.2],a);
                r.cube([p.x,p.y+1.8,p.z],[3.2,2.3,7.8],if self.heists.gold(wagon){[0.5,0.4,0.17]}else{[0.18,0.35,0.45]},a);
                continue;
            }
            r.rounded(
                [p.x, p.y, p.z],
                [3.5, 1.4, 8.5],
                [0.12, 0.16, 0.2],
                a,
                8,
                0.35,
            );
            let gold = self.heists.gold(wagon);
            if self.heists.site == Some(5)
                && self.heists.riding == Some(wagon)
                && self.heists.opened
            {
                r.cube([p.x, p.y + 0.8, p.z], [3.2, 0.3, 7.8], [0.25, 0.3, 0.33], a);
                r.rounded(
                    [p.x, p.y + 1.6, p.z],
                    [1.7, 1.4, 2.2],
                    [0.3, 0.35, 0.38],
                    a,
                    8,
                    0.3,
                );
            } else {
                r.cube(
                    [p.x, p.y + 1.8, p.z],
                    [3.2, 2.3, 7.8],
                    if gold {
                        [0.5, 0.4, 0.17]
                    } else {
                        [0.18, 0.35, 0.45]
                    },
                    a,
                );
            }
            thread_local! { static RUNNING_GEAR: engine_core::Mesh = vehicles::rail::running_gear(); }
            RUNNING_GEAR.with(|mesh| r.solid_model(mesh, [p.x, p.y, p.z], a, [1.; 3]));
            // The opened robbery wagon keeps its accessible deck unobstructed.
            if !(self.heists.site == Some(5) && self.heists.riding == Some(wagon) && self.heists.opened) {
                thread_local! { static EQUIPMENT: engine_core::Mesh = vehicles::rail::wagon_equipment(); }
                EQUIPMENT.with(|mesh| r.solid_model(mesh, [p.x, p.y, p.z], a, [1.; 3]));
            }
            if gold {
                r.cube([p.x, p.y + 3., p.z], [1.5, 0.3, 2.], [0.95, 0.75, 0.18], a);
            }
        }
        let (p, a) = crate::heists::Heists::train(self.clock, 0);
        let q = Vec3::new(p.x + a.sin() * 10., p.y, p.z + a.cos() * 10.);
        if q.distance(self.player.pos) < quality.distance && visible(q) {
            thread_local! {
                static LOCOMOTIVES: [engine_core::Mesh; 2] = [vehicles::rail::locomotive(false), vehicles::rail::locomotive(true)];
            }
            LOCOMOTIVES.with(|meshes| r.solid_model(&meshes[usize::from(q.distance(self.player.pos) < quality.detail)], [q.x, q.y, q.z], a, [1.; 3]));
        }
        // Casino retains its existing security beams; J-Store uses the shared layout above.
        let s=crate::heists::SITES[3];
        let floor=self.world.surface(s.x,s.z);
        if self.player.pos.distance(Vec3::new(s.x,floor+1.,s.z))<=80. {
            for i in 0..5 {
                r.cube([s.x,floor+0.8,s.z-8.+i as f32*4.],[22.,0.06,0.06],[1.,0.12,0.03],0.);
            }
        }
        for (p, taken) in &self.drops {
            if !taken && p.distance(self.player.pos) < 70. {
                r.cube(
                    [p.x, p.y - 0.6, p.z],
                    [0.6, 0.25, 0.4],
                    [0.4, 0.5, 0.22],
                    0.,
                );
            }
        }
        if self.parachute
            && self.player.pos.y > self.world.surface(self.player.pos.x, self.player.pos.z) + 3.
        {
            let p = self.player.pos;
            r.rounded(
                [p.x, p.y + 4., p.z],
                [7., 1.2, 4.],
                [0.9, 0.4, 0.1],
                self.player.heading,
                8,
                0.7,
            );
            for side in [-1., 1.] {
                r.cube(
                    [p.x + side * 1.3, p.y + 2., p.z],
                    [0.03, 4., 0.03],
                    WHITE,
                    0.,
                );
            }
        }
        if self.player.pos.distance(Vec3::new(320., 1., 35.)) < 70. {
            for z in [20., 24., 28.] {
                r.cube([320., 0.8, z], [14., 0.06, 0.06], [1., 0.1, 0.05], 0.);
            }
        }
        let y = self.world.surface(-290., 100.);
        if self.player.pos.distance(Vec3::new(-290., y + 1., 100.)) < 70. {
            draw_person(
                r,
                Vec3::new(-290., y + 1., 100.),
                0.,
                [0.15, 0.13, 0.15],
                0.,
                false,
                0,
                0.,
                0.,
                0.,
                0,
            );
        }
    }
    fn draw_heist_ui(&self, r: &mut Renderer) {
        if let Some(id) = self.world.interior(self.player.pos) {
            let b = &self.world.buildings[id];
            if matches!(b.name, "Office" | "Westhaven" | "Jewelry" | "Casino") {
                text(
                    r,
                    300.,
                    470.,
                    1.8,
                    &if b.name=="Jewelry" && self.heists.bag>0 {
                        "J-STORE: SIDE RAMPS TO ROOF; P PARACHUTE; DELIVER LOOT".to_string()
                    } else {format!(
                        "FLOOR {} - E AT REAR LIFT; P PARACHUTE",
                        ((self.player.pos.y - b.p.y - 0.6) / 4.).floor() as u32 + 1
                    )},
                    GOLD,
                );
            }
        }
        text(
            r,
            32.,
            146.,
            1.6,
            &format!(
                "BOUNTY ${}  {}",
                self.bounty,
                "*".repeat(self.wanted.min(5) as usize)
            ),
            GOLD,
        );
        if let Some(id) = self.heists.site {
            let s = crate::heists::SITES[id];
            let delay = if id == 3 {
                300. - self.wanted.min(5) as f32 * 40.
            } else {
                s.delay / (1. + self.wanted.saturating_sub(1) as f32 * 0.12)
            };
            r.panel([720., 260., 280., 105.], INK, 0.9, 18.);
            text(r, 735., 275., 2., if id==0 {"J-STORE"} else {s.name}, GOLD);
            text(
                r,
                735.,
                305.,
                1.7,
                &format!("POLICE {:.0}s", (delay - self.heists.elapsed).max(0.)),
                WHITE,
            );
            text(
                r,
                735.,
                332.,
                1.7,
                &format!(
                    "BAG {:.0}%  ${}",
                    (self.heists.progress / 8. * 100.).min(100.),
                    self.heists.bag
                ),
                WHITE,
            );
        }
        if self.heists.quiz {
            r.panel([235., 190., 560., 205.], INK, 0.98, 22.);
            text(r, 260., 210., 2.5, "SECURITY TERMINAL - SOLVE x", GOLD);
            text(r, 260., 262., 3., &self.heists.expression, WHITE);
            text(
                r,
                260.,
                307.,
                2.5,
                &format!("x = {}", self.heists.answer),
                BLUE,
            );
            text(
                r,
                260.,
                355.,
                1.7,
                "TYPE ANSWER / ENTER; BACKSPACE TO EDIT",
                WHITE,
            );
        }
    }
}

impl Game {
    fn drop_c4(&mut self, id: usize) {
        if self.npcs[id].health > 0. {
            return;
        }
        if self.player.team != Team::Police {
            self.wanted = (self.wanted + 1).min(5);
            self.bounty += 150;
        }
        self.random = self
            .random
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        if (self.random >> 32) % 100 < 30 {
            self.drops.push((self.npcs[id].pos, false));
        }
        self.npcs[id].disabled = 45.;
    }
}

const PAINTS: [[f32; 3]; 8] = [
    [0.95, 0.6, 0.08],
    [0.05, 0.7, 0.85],
    [0.65, 0.12, 0.85],
    [0.92, 0.16, 0.12],
    [0.15, 0.8, 0.4],
    [0.95, 0.95, 0.93],
    [0.08, 0.09, 0.13],
    [0.85, 0.22, 0.55],
];
impl Game {
    fn set_engine(&mut self, target: u32) {
        let Some(id) = self.workshop else {
            return;
        };
        if target > 5 {
            return;
        }
        let level = self.save.car_engine[self.cars[id].kind];
        let cost = (level..target).map(|n| 150 + n * 100).sum::<u32>();
        if self.save.balance < cost {
            self.notify("NOT ENOUGH CASH FOR SELECTED ENGINE");
            return;
        }
        self.save.balance -= cost;
        self.cars[id].engine_level = target;
        self.save.car_engine[self.cars[id].kind] = level.max(target);
        self.save.car_engine_selected[self.cars[id].kind] = target;
        self.notify("ENGINE CONFIGURED - ENTER TO SAVE PRESET");
    }
    fn draw_customs(&self, r: &mut Renderer) {
        modal_title(r, "CUSTOMS - SELECT YOUR CONFIGURATION");
        let id = self.workshop.unwrap();
        let c = &self.cars[id];
        text(
            r,
            210.,
            195.,
            2.,
            &format!("{} / {} - {}", id + 1, CATALOG.len(), CATALOG[c.kind].name),
            WHITE,
        );
        gui::button(r, [210., 224., 140., 35.], "< PREVIOUS", BLUE);
        gui::button(r, [365., 224., 140., 35.], "NEXT >", BLUE);
        text(r, 210., 278., 1.8, "SELECT ENGINE LEVEL (0-5)", GOLD);
        for level in 0..=5 {
            gui::button(
                r,
                [210. + level as f32 * 98., 302., 85., 40.],
                &format!("LV {level}"),
                if c.engine_level == level {
                    ORANGE
                } else {
                    BLUE
                },
            );
            let cost = (self.save.car_engine[c.kind]..level)
                .map(|n| 150 + n * 100)
                .sum::<u32>();
            text(
                r,
                220. + level as f32 * 98.,
                344.,
                1.4,
                &format!("${cost}"),
                WHITE,
            );
        }
        text(r, 210., 366., 1.8, "SPECIAL PAINT $25", WHITE);
        for (i, color) in PAINTS.iter().enumerate() {
            gui::button(
                r,
                [210. + i as f32 * 73., 383., 62., 38.],
                &format!("{}", i + 1),
                *color,
            );
        }
        gui::button(
            r,
            [210., 439., 285., 35.],
            "ENTER - SAVE PRESET",
            [0.1, 0.6, 0.35],
        );
        gui::button(r, [515., 439., 285., 35.], "BACK TO SERVICE", INK);
    }
    fn customs_click(&mut self, x: f32, y: f32) -> bool {
        if self.workshop.is_none() {
            return false;
        }
        if !self.workshop_config {
            if gui::contains([530., 444., 280., 28.], x, y) {
                self.workshop_config = true;
                return true;
            }
            return false;
        }
        let id = self.workshop.unwrap();
        if gui::contains([210., 224., 140., 35.], x, y) {
            self.workshop = Some((id + CATALOG.len() - 1) % CATALOG.len());
        }
        if gui::contains([365., 224., 140., 35.], x, y) {
            self.workshop = Some((id + 1) % CATALOG.len());
        }
        for level in 0..=5 {
            if gui::contains([210. + level as f32 * 98., 302., 85., 40.], x, y) {
                self.set_engine(level);
            }
        }
        for (i, color) in PAINTS.iter().enumerate() {
            if gui::contains([210. + i as f32 * 73., 383., 62., 38.], x, y) {
                if self.save.balance >= 25 {
                    self.save.balance -= 25;
                    self.cars[id].color = *color;
                    self.save.car_paint[self.cars[id].kind] = *color;
                } else {
                    self.notify("NOT ENOUGH CASH");
                }
            }
        }
        if gui::contains([210., 439., 285., 35.], x, y) {
            self.save();
        }
        if gui::contains([515., 439., 285., 35.], x, y) {
            self.workshop_config = false;
        }
        true
    }
}
#[cfg(test)]
mod expansion_tests {
    use super::*;
    fn game() -> Game {
        let mut g = Game::from_save(Default::default()).unwrap();
        g.menu = false;
        g.npcs.clear();
        g.player.team = Team::Criminal;
        for d in &mut g.doors {
            d.open = true;
            d.angle = std::f32::consts::FRAC_PI_2;
        }
        g
    }
    #[test]
    fn jewelry_ramps_are_walkable_through_real_player_simulation() {
        let mut g=game();
        g.npcs.clear();
        let b=g.world.buildings.iter().find(|b|b.name=="Jewelry").unwrap().clone();
        for ramp in buildings::jewelry::RAMPS {
            g.player.pos=Vec3::new(b.p.x+ramp.start[0],b.p.y+ramp.start[1]+1.,b.p.z+ramp.start[2]);
            g.player.vy=0.;
            let dx=ramp.end[0]-ramp.start[0];let dz=ramp.end[2]-ramp.start[2];
            g.yaw=dx.atan2(-dz);
            for _ in 0..200 {g.simulate(1.,0.,false,false,false);}
            assert!((g.player.pos.y-b.p.y-ramp.end[1]-1.).abs()<0.15,"ramp {:?}: y {}",ramp.end,g.player.pos.y);
            assert!(g.player.pos.distance(Vec3::new(b.p.x+ramp.end[0],g.player.pos.y,b.p.z+ramp.end[2]))<0.3);
        }
    }
    #[test]
    fn jewelry_upper_floors_cannot_break_ground_displays_or_collect_early() {
        let mut g=game();g.npcs.clear();g.heists.start(0);
        g.player.pos=Vec3::new(254.,13.,190.);
        for _ in 0..3 {g.special_strike();}
        assert_eq!(g.heists.strikes,0);
        g.player.pos=Vec3::new(260.,5.,190.);g.holding=true;
        for _ in 0..481 {g.tick_heists();}
        assert_eq!(g.heists.bag,0);
        assert_eq!(g.heists.progress,0.);
    }
    #[test]
    fn jewelry_laser_damage_matches_jump_crouch_and_alarm_state() {
        let mut g=game();g.npcs.clear();
        g.player.pos=Vec3::new(260.,1.,184.);
        g.tick_heists();assert_eq!(g.player.health,100.);
        g.heists.start(0);g.tick_heists();assert!(g.player.health<100.);
        g.player.health=100.;g.player.pos.y=2.;g.tick_heists();assert_eq!(g.player.health,100.);
        g.player.pos=Vec3::new(260.,9.,188.);g.stance=1;g.tick_heists();assert_eq!(g.player.health,100.);
        g.stance=0;g.tick_heists();assert!(g.player.health<100.);
    }

    #[test]
    fn jewelry_has_storage_roof_and_collector_pays_once() {
        let mut g = game();
        g.player.pos = Vec3::new(254., 1., 190.);
        assert!(g.special_interact());
        assert_eq!(g.heists.site, Some(0));
        for _ in 0..3 {
            assert!(g.special_strike());
        }
        g.player.pos = Vec3::new(260., 1., 182.);
        g.special_interact();
        assert_eq!(g.player.pos.y, 5.);
        g.special_interact();
        assert_eq!(g.player.pos.y, 9.);
        g.special_interact();
        assert_eq!(g.player.pos.y, 13.);
        assert_eq!(g.world.walking_surface(g.player.pos), 12.);
        g.special_interact();
        g.special_interact();
        assert_eq!(g.player.pos.y, 5.);
        g.player.pos.z = 190.;
        g.holding = true;
        for _ in 0..481 {
            g.tick_heists();
        }
        assert!(g.heists.bag > 0);
        assert_eq!(g.save.balance, 0);
        g.player.pos = Vec3::new(-290., 1., 100.);
        assert!(g.special_interact());
        assert_eq!(g.save.balance,0);
        assert!(g.heists.bag>0);
        g.player.pos=Vec3::new(260.,13.,182.);
        g.tick_heists();
        assert!(g.heists.jewelry_roof_reached);
        g.player.pos = Vec3::new(-290., 1., 100.);
        assert!(g.special_interact());
        let paid = g.save.balance;
        assert!(paid > 0);
        g.special_interact();
        assert_eq!(paid, g.save.balance);
        assert!((300. ..=900.).contains(&g.heists.cooldown[0]));
    }
    #[test]
    fn casino_requires_five_answers_and_first_floor_loot() {
        let mut g = game();
        g.player.pos = Vec3::new(382., 9., 252.);
        assert!(g.special_interact());
        assert!(g.heists.quiz);
        for _ in 0..5 {
            g.heists.answer = g.heists.expected.to_string();
            assert!(g.heists.submit());
        }
        g.holding = true;
        g.tick_heists();
        assert_eq!(g.heists.progress, 0.);
        g.player.pos = Vec3::new(370., 1., 241.);
        g.tick_heists();
        assert!(g.heists.progress > 0.);
    }
    #[test]
    fn immunity_blocks_taser_and_arrest_but_expires() {
        let mut g = game();
        g.wanted = 3;
        g.player.pos = Vec3::new(110., 1., 236.);
        g.npcs = vec![Npc::new(Role::Police, 110., 235.)];
        g.immunity = 12.;
        g.tick_npcs();
        assert_eq!(g.stun, 0.);
        assert!(!g.cuffs.active);
        g.immunity = 0.;
        g.tick_npcs();
        assert!(g.cuffs.active);
    }
    #[test]
    fn direct_engine_upgrade_is_atomic_and_reaches_five() {
        let mut g = game();
        g.workshop = Some(0);
        g.save.balance = 1000;
        g.set_engine(5);
        assert_eq!(g.cars[0].engine_level, 0);
        assert_eq!(g.save.balance, 1000);
        g.save.balance = 2000;
        g.set_engine(5);
        assert_eq!(g.cars[0].engine_level, 5);
        assert_eq!(g.save.balance, 250);
        assert_eq!(g.save.car_engine[0], 5);
        g.set_engine(0);
        g.set_engine(5);
        assert_eq!(g.save.balance, 250);
        assert_eq!(g.save.car_engine_selected[0], 5);
    }
    #[test]
    fn train_requires_c4_and_questions_dont_accept_wrong_answers() {
        let mut g = game();
        let w = g.heists.train_gold[0];
        g.heists.start(5);
        g.heists.riding = Some(w);
        g.slot = 2;
        assert!(g.special_interact());
        assert!(!g.heists.opened);
        g.heists.riding = Some(w);
        g.explosive = true;
        assert!(g.special_interact());
        assert!(!g.explosive);
        for _ in 0..181 {
            g.heists.tick(STEP);
        }
        assert!(g.heists.opened);
    }
}
