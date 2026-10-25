//! Procedural interpretations of the user's absurdity list. No runtime heap growth.
use super::{direction, dot, finite, sub, PhysicsSaboteur, Rng, BRIDGE_CAPACITY};
use crate::npcs::Npc;
use engine_core::{Mesh, Vec3};
use renderer_gl21::{GpuMesh, Renderer, Texture};
use vehicles::Car;
use world::World;

const ACTORS: usize = 96;
const PIGEONS: std::ops::Range<usize> = 32..96;
macro_rules! models {
    ($($name:ident),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Debug)]
        #[repr(usize)]
        enum Model { $($name),+ }
        const MODELS: &[Model] = &[$(Model::$name),+];
    };
}
models!(
    Titan,
    Dumpling,
    Wardrobe,
    Fridge,
    Banana,
    Borscht,
    Toilet,
    Chair,
    Cow,
    Granny,
    Rotor,
    Umbrella,
    Sign,
    Tv,
    Centaur,
    ErrorHead,
    Sausage,
    Ping,
    Square,
    Triangle,
    Cucumber,
    Grandpa,
    Atm,
    Kettle,
    Bus,
    Cursor,
    Cat,
    Washer,
    Pigeon,
    Duck,
    Sofa,
    Note,
    Watermelon,
    Clock,
    Brick,
    Worm,
    PoliceFridge,
    Jpeg,
    Cabbage,
    Globe,
    FKey,
    Portal,
    Generator,
    King,
    Toaster,
    Crosswalk,
    Printer,
    Mushroom,
    Foot,
    Bread,
    Pine,
    Admin,
    Stool,
    ToiletArmy,
    GrannyArmy,
    WasherParade,
    Bodyguards
);

#[derive(Clone, Copy, PartialEq)]
enum Motion {
    Still,
    Walk,
    Chase,
    Orbit,
    Fly,
    Rain,
    Fall,
    Escape,
}

#[derive(Clone, Copy)]
struct Scene {
    id: u8,
    title: &'static str,
    caption: &'static str,
    model: Model,
    motion: Motion,
    scale: f32,
    count: u8,
    audio: u8,
}
macro_rules! scene {
    ($id:literal,$title:literal,$caption:literal,$model:ident,$motion:ident,$scale:literal,$count:literal,$audio:literal) => {
        Scene {
            id: $id,
            title: $title,
            caption: $caption,
            model: Model::$model,
            motion: Motion::$motion,
            scale: $scale,
            count: $count,
            audio: $audio,
        }
    };
}
const SCENES: &[Scene] = &[
    scene!(
        1,
        "ПРОПЕЛЕРНИЙ ПАТРУЛЬ",
        "180 КМ/ГОД ЗАДОМ НАПЕРЕД",
        Titan,
        Chase,
        2.5,
        8,
        4
    ),
    scene!(
        2,
        "ПЕЛЬМЕНЬ ПИТАЄ ЧАС",
        "КОТРА ГОДИНА?",
        Dumpling,
        Walk,
        15.0,
        1,
        0
    ),
    scene!(
        3,
        "ДОЩ ІЗ ХОЛОДИЛЬНИКІВ",
        "УСЕРЕДИНІ ТРИВАЄ ШАХОВИЙ ТУРНІР",
        Fridge,
        Rain,
        1.0,
        24,
        3
    ),
    scene!(
        4,
        "СУДДЯ-БАНАН",
        "ВИРОК: КОЛЕСА НЕ ТОГО КОЛЬОРУ",
        Banana,
        Still,
        3.0,
        1,
        3
    ),
    scene!(
        5,
        "ГРАВІТАЦІЙНИЙ БОРЩ",
        "ГРАВІТАЦІЯ ВИМКНЕНА НА 8 СЕКУНД",
        Borscht,
        Fly,
        9.0,
        1,
        1
    ),
    scene!(
        6,
        "ОРКЕСТР УНІТАЗІВ",
        "400 УНІТАЗІВ. ЖОДНОГО СЕНСУ.",
        ToiletArmy,
        Walk,
        1.0,
        1,
        3
    ),
    scene!(7, "БОДЯ НА ОРБІТІ", "ЦЕ RTX 5090", Chair, Orbit, 2.0, 1, 1),
    scene!(
        8,
        "ШІСТНАДЦЯТИНОГА КОРОВА",
        "300 КМ/ГОД. КОЖНА НОГА ОКРЕМО.",
        Cow,
        Orbit,
        1.5,
        1,
        5
    ),
    scene!(
        9,
        "БАБУСЯ-ВЕРТОЛІТ",
        "ВАРЕНИКИ З ДОСТАВКОЮ. E КУПИТИ.",
        Granny,
        Fly,
        2.0,
        1,
        4
    ),
    scene!(
        10,
        "ЗНАК ПЕЛЬМЕНІ",
        "ДИВИСЬ НА ЗНАК: УСІ NPC ПРИСЯДУТЬ",
        Sign,
        Still,
        3.0,
        1,
        0
    ),
    scene!(
        11,
        "НЕБЕСНИЙ ТЕЛЕВІЗОР",
        "LIVE: ТЕЛЕВІЗОР ДИВИТЬСЯ НА СЕБЕ",
        Tv,
        Fly,
        60.0,
        1,
        1
    ),
    scene!(
        13,
        "ПРАЛЬНИЙ КЕНТАВР",
        "ПРОГРАМА: БІЦЕПСИ 90 ГРАДУСІВ",
        Centaur,
        Chase,
        2.5,
        2,
        3
    ),
    scene!(
        15,
        "ЛЮДИНА СИСТЕМНОЇ ПОМИЛКИ",
        "SYSTEM ERROR: ОСОБИСТІСТЬ НЕ ЗНАЙДЕНО",
        ErrorHead,
        Walk,
        1.5,
        3,
        5
    ),
    scene!(
        19,
        "ЛІТАЮЧА КОВБАСА",
        "MODEM CONNECTING... 1998",
        Sausage,
        Fly,
        35.0,
        1,
        1
    ),
    scene!(21, "ЛЮДИНА-ПІНГ", "999999ms", Ping, Walk, 1.5, 3, 3),
    scene!(
        24,
        "КВАДРАТ ІЗ БІЦЕПСАМИ",
        "ЧОРНИЙ КВАДРАТ ПОЛЮЄ НА РОЖЕВЕ",
        Square,
        Chase,
        2.0,
        3,
        4
    ),
    scene!(
        26,
        "ДІД ІЗ ЧОТИРМА RTX",
        "9000 FPS У САПЕРІ. E ПЕРЕВІРИТИ.",
        Grandpa,
        Still,
        1.7,
        1,
        3
    ),
    scene!(
        31,
        "КАЧКА З ДИПЛОМОМ",
        "КВАНТОВІ МАКАРОНИ З АВТОМОБІЛІВ",
        Duck,
        Still,
        4.0,
        1,
        2
    ),
    scene!(
        32,
        "КАРТОПЛЯНА РУЛЕТКА",
        "КОЖНА ДЕСЯТА МОДЕЛЬ: КАРТОПЛЯ З ВУСАМИ",
        Sign,
        Still,
        2.0,
        1,
        0
    ),
    scene!(
        36,
        "АРМІЯ БАБУСЬ",
        "ТИ ПОЇВ? 300 БАБУСЬ У T-POSE.",
        GrannyArmy,
        Chase,
        1.0,
        1,
        2
    ),
    scene!(
        37,
        "БАНКОМАТ ІЗ ДЕПРЕСІЄЮ",
        "E: ОТРИМАТИ SKILL ISSUE",
        Atm,
        Still,
        2.0,
        1,
        3
    ),
    scene!(
        44,
        "ТЕЛЕПОРТАЦІЙНА СОСИСКА",
        "КОЖНІ 5 СЕКУНД МІНЯЄТЬСЯ МІСЦЯМИ З NPC",
        Sausage,
        Still,
        3.0,
        1,
        1
    ),
    scene!(
        45,
        "ЧАЙНИК АСЕРТІВ",
        "ASSERTION FAILED!",
        Kettle,
        Chase,
        4.0,
        1,
        4
    ),
    scene!(
        48,
        "МАРШРУТ У НІКУДИ",
        "МАРШРУТ № NaN. ЗУПИНКА БІЛЯ КАЛЮЖІ.",
        Bus,
        Orbit,
        3.0,
        1,
        3
    ),
    scene!(
        51,
        "КУРСОР АДМІНІСТРАТОРА",
        "ПРАВА КНОПКА ПО МІСТУ",
        Cursor,
        Fly,
        25.0,
        1,
        3
    ),
    scene!(
        52,
        "КІТ PENTIUM 4",
        "CPU 99 C. ПРОЦЕС НЕ ВІДПОВІДАЄ.",
        Cat,
        Walk,
        2.0,
        1,
        3
    ),
    scene!(
        53,
        "ПАРАД ПРАЛЬНИХ МАШИН",
        "90 ПРАЛОК ВИКОНУЮТЬ ДРИФТ",
        WasherParade,
        Orbit,
        1.0,
        1,
        3
    ),
    scene!(
        57,
        "ГОЛУБ-БУХГАЛТЕР",
        "СПИСАНО: 0 ДОЛАРІВ",
        Pigeon,
        Orbit,
        2.0,
        1,
        2
    ),
    scene!(
        58,
        "БОЙОВИЙ ДИВАН",
        "ВІДПОЧИНЬ! E: ПРИСІСТИ.",
        Sofa,
        Chase,
        2.0,
        1,
        4
    ),
    scene!(
        59,
        "ОРКЕСТР НЕВИДИМИХ ЖАБ",
        "КВАКАННЯ У ПРОСТОРОВОМУ ЗВУЦІ",
        Note,
        Fly,
        2.0,
        12,
        2
    ),
    scene!(
        64,
        "КАВУН ІЗ WI-FI",
        "PING: -500ms",
        Watermelon,
        Still,
        8.0,
        1,
        1
    ),
    scene!(
        68,
        "ПРИВИД CRT",
        "СТІНИ НЕ ІСНУЮТЬ. ДВЕРІ ЗАНАДТО ВУЗЬКІ.",
        ErrorHead,
        Walk,
        2.5,
        3,
        5
    ),
    scene!(
        71,
        "ГОДИННИК ІЗ 73 СТРІЛКАМИ",
        "НОВИЙ РІК КОЖНІ 14 СЕКУНД",
        Clock,
        Orbit,
        6.0,
        1,
        5
    ),
    scene!(72, "ЦЕГЛИНА-ЮРИСТ", "Я НЕ КАМІНЬ!", Brick, Chase, 2.0, 1, 3),
    scene!(
        74,
        "М'ЯЗИСТИЙ ЧЕРВ'ЯК",
        "ПРОТЕЇН ПІД АСФАЛЬТОМ",
        Worm,
        Still,
        6.0,
        1,
        4
    ),
    scene!(
        75,
        "ХОЛОДИЛЬНИЙ СПЕЦНАЗ",
        "ЗАМІСТЬ СИРЕНИ: МІКРОХВИЛЬОВКА",
        PoliceFridge,
        Orbit,
        2.0,
        6,
        3
    ),
    scene!(
        76,
        "JPEG-ОПАДИ",
        "ХМАРИ 16x16. ЯКІСТЬ: МІНУС.",
        Jpeg,
        Rain,
        1.0,
        24,
        1
    ),
    scene!(
        77,
        "КАПУСТА ПІД ОХОРОНОЮ",
        "12 ОХОРОНЦІВ. ОДНА КАПУСТИНА.",
        Bodyguards,
        Walk,
        1.0,
        1,
        0
    ),
    scene!(78, "ПЛОСКИЙ ГЛОБУС", "Я ПЛОСКИЙ", Globe, Walk, 5.0, 1, 3),
    scene!(
        80,
        "ВЕЛИКА КЛАВІША F",
        "УСІ NPC ВІДДАЮТЬ ШАНУ",
        FKey,
        Fall,
        6.0,
        1,
        4
    ),
    scene!(
        82,
        "ПОРТАЛ У БУХГАЛТЕРІЮ",
        "E: ЗАЙТИ В ОФІС",
        Portal,
        Still,
        2.0,
        1,
        1
    ),
    scene!(
        84,
        "МАРМЕЛАДНИЙ НЕБОСХИЛ",
        "НЕБО ПЕРЕТВОРИЛОСЯ НА ЗЕЛЕНЕ ЖЕЛЕ",
        Borscht,
        Fly,
        15.0,
        1,
        2
    ),
    scene!(
        85,
        "КУРЯЧИЙ ВІЧНИЙ ДВИГУН",
        "ДВІ КУРКИ ЖИВЛЯТЬ ОДНУ ЛАМПУ",
        Generator,
        Still,
        3.0,
        1,
        4
    ),
    scene!(
        86,
        "РАЙДУЖНА ШАФА",
        "ВАМ ЗАПРОПОНУВАТИ ШКАРПЕТКИ?",
        Wardrobe,
        Chase,
        2.5,
        2,
        3
    ),
    scene!(
        89,
        "КОРОЛЬ МІКРОХВИЛЬОВОК",
        "ТОСТЕРИ, ДО БОЮ!",
        King,
        Still,
        4.0,
        1,
        3
    ),
    scene!(
        90,
        "ПЕРЕХІД, ЯКИЙ ТІКАЄ",
        "ЗЕБРА НЕ ХОЧЕ, ЩОБ ЇЇ ПЕРЕХОДИЛИ",
        Crosswalk,
        Escape,
        3.0,
        1,
        4
    ),
    scene!(
        94,
        "ПРИНТЕР НЕСКІНЧЕННИХ ГОЛУБІВ",
        "30 ГОЛУБІВ ЗА СЕКУНДУ",
        Printer,
        Still,
        2.0,
        1,
        3
    ),
    scene!(
        95,
        "МЕР: ВОСЬМИБІТНИЙ ГРИБ",
        "УСІ ДОРОГИ ВІДТЕПЕР ВЕРТИКАЛЬНІ",
        Mushroom,
        Still,
        12.0,
        1,
        5
    ),
    scene!(
        96,
        "ВЕЛИКА ГУМОВА НОГА",
        "ПИЩАЛКА МАСШТАБУ МІСТА",
        Foot,
        Still,
        35.0,
        1,
        4
    ),
    scene!(
        97,
        "ХЛІБНИЙ СУПУТНИК",
        "РАДІО: РЕЦЕПТ БЕТОНУ",
        Bread,
        Orbit,
        18.0,
        1,
        1
    ),
    scene!(
        99,
        "АНТИГРАВІТАЦІЙНА СОСНА",
        "КОРІННЯ ВГОРУ. КРОСІВКИ НА ГІЛКАХ.",
        Pine,
        Fly,
        5.0,
        1,
        0
    ),
    scene!(
        100,
        "ГОЛОВНИЙ АДМІН ВСЕСВІТУ",
        "404: MONDAY NOT FOUND",
        Admin,
        Fly,
        45.0,
        1,
        5
    ),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interaction {
    Message(&'static str),
    Rest,
    BuyDumplings,
    Portal,
}

pub struct AbsurdDirector {
    meshes: [Option<GpuMesh>; MODELS.len()],
    potato: Option<Texture>,
    active: [bool; ACTORS],
    model: [Model; ACTORS],
    position: [Vec3; ACTORS],
    velocity: [Vec3; ACTORS],
    scale: [f32; ACTORS],
    yaw: [f32; ACTORS],
    center: Vec3,
    scene: Option<usize>,
    next: f32,
    remaining: f32,
    elapsed: f32,
    clock: f32,
    swap: f32,
    printer: f32,
    pigeon_cursor: usize,
    crouch: bool,
    lift_player: bool,
    started: bool,
    automatic: bool,
    orbit_radius: f32,
}
impl AbsurdDirector {
    pub fn new() -> Self {
        Self {
            meshes: [None; MODELS.len()],
            potato: None,
            active: [false; ACTORS],
            model: [Model::Dumpling; ACTORS],
            position: [Vec3::default(); ACTORS],
            velocity: [Vec3::default(); ACTORS],
            scale: [1.; ACTORS],
            yaw: [0.; ACTORS],
            center: Vec3::default(),
            scene: None,
            next: 3.,
            remaining: 0.,
            elapsed: 0.,
            clock: 0.,
            swap: 5.,
            printer: 0.,
            pigeon_cursor: PIGEONS.start,
            crouch: false,
            lift_player: false,
            started: false,
            automatic: true,
            orbit_radius: 100.,
        }
    }
    pub fn initialize(&mut self, r: &mut Renderer) {
        if self.potato.is_some() {
            return;
        }
        self.potato = Some(r.texture(16, 16, &potato_pixels()));
        for &model in MODELS {
            self.meshes[model as usize] = Some(r.upload(&build_model(model)));
        }
    }
    #[cfg(test)]
    pub fn disable_automatic(&mut self) {
        self.automatic = false;
    }
    pub fn scene_count() -> usize {
        SCENES.len()
    }
    pub fn source_id(&self) -> u8 {
        self.scene.map_or(0, |i| SCENES[i].id)
    }
    pub fn gravity_off(&self) -> bool {
        self.source_id() == 5 && self.elapsed < 8. && self.remaining > 0.
    }
    pub fn sky_jelly(&self) -> bool {
        matches!(self.source_id(), 84 | 100) && self.remaining > 0.
    }
    pub fn crouch(&self) -> bool {
        self.crouch && self.remaining > 0.
    }
    pub fn stools(&self) -> bool {
        self.source_id() == 100 && self.elapsed > 5. && self.remaining > 0.
    }
    pub fn noodle_cars(&self) -> bool {
        self.source_id() == 31 && self.remaining > 0.
    }
    pub fn audio(&self) -> u8 {
        if self.remaining > 0. {
            self.scene.map_or(0, |i| SCENES[i].audio)
        } else {
            0
        }
    }
    pub fn potato_texture(&self, i: usize) -> Option<Texture> {
        if self.source_id() == 32 && self.remaining > 0. && i.is_multiple_of(10) {
            self.potato
        } else {
            None
        }
    }
    pub fn npc_mesh(&self) -> Option<GpuMesh> {
        if self.stools() {
            self.meshes[Model::Stool as usize]
        } else {
            None
        }
    }
    pub fn take_player_lift(&mut self) -> bool {
        std::mem::take(&mut self.lift_player)
    }
    pub fn advance(&mut self, world: &World, p: Vec3, forward: Vec3, rng: &mut Rng) {
        let mut next = rng.below(SCENES.len() as u32) as usize;
        if self.scene == Some(next) {
            next = (next + 1) % SCENES.len();
        }
        self.force(SCENES[next].id, world, p, forward);
    }
    fn activate(&mut self, i: usize, model: Model, p: Vec3, scale: f32) {
        self.active[i] = true;
        self.model[i] = model;
        self.position[i] = p;
        self.scale[i] = scale;
        self.velocity[i] = Vec3::default();
        self.yaw[i] = 0.;
    }
    pub fn force(&mut self, id: u8, world: &World, player: Vec3, forward: Vec3) -> bool {
        let Some(index) = SCENES.iter().position(|s| s.id == id) else {
            return false;
        };
        self.active.fill(false);
        self.scene = Some(index);
        self.elapsed = 0.;
        self.remaining = 20.;
        self.next = 26.;
        self.swap = 5.;
        self.printer = 0.;
        self.started = true;
        self.lift_player = false;
        self.crouch = false;
        let mut horizontal = forward;
        horizontal.y = 0.;
        horizontal = direction(horizontal);
        if dot(horizontal, horizontal) < 0.1 {
            horizontal = Vec3::new(0., 0., 1.);
        }
        self.center = Vec3::new(
            player.x + horizontal.x * 26.,
            0.,
            player.z + horizontal.z * 26.,
        );
        self.center.y = world.surface(self.center.x, self.center.z);
        self.orbit_radius = player.x.hypot(player.z).max(80.);
        let scene = SCENES[index];
        for i in 0..scene.count as usize {
            let a = i as f32 * 2.399_963;
            let radius = if scene.count > 1 {
                5. + (i as f32).sqrt() * 2.5
            } else {
                0.
            };
            let mut p = Vec3::new(
                self.center.x + a.cos() * radius,
                self.center.y + scene.scale,
                self.center.z + a.sin() * radius,
            );
            if matches!(scene.motion, Motion::Rain | Motion::Fall) {
                p.y = 45. + i as f32 * 2.;
            }
            if scene.motion == Motion::Fly {
                p.y += 8.;
            }
            if id == 100 {
                p.y = 90.;
            }
            self.activate(i, scene.model, p, scene.scale);
        }
        if id == 24 {
            for i in 3..7 {
                self.activate(
                    i,
                    Model::Triangle,
                    Vec3::new(
                        self.center.x + i as f32 * 3.,
                        self.center.y + 1.,
                        self.center.z + 8.,
                    ),
                    1.5,
                );
            }
        }
        if id == 89 {
            for i in 1..25 {
                let a = i as f32 * 0.7;
                self.activate(
                    i,
                    Model::Toaster,
                    Vec3::new(
                        self.center.x + a.cos() * 8.,
                        self.center.y + 0.5,
                        self.center.z + a.sin() * 8.,
                    ),
                    0.7,
                );
            }
        }
        true
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        world: &World,
        cars: &mut [Car],
        npcs: &mut [Npc],
        player: Vec3,
        eye: Vec3,
        forward: Vec3,
        physics: &mut PhysicsSaboteur,
        rng: &mut Rng,
        dt: f32,
    ) {
        self.clock = (self.clock + dt).rem_euclid(3600.);
        self.elapsed += dt;
        self.remaining = (self.remaining - dt).max(0.);
        self.next -= dt;
        if self.next <= 0. && self.automatic {
            let old = self.scene;
            let mut choice = rng.below(SCENES.len() as u32) as usize;
            if old == Some(choice) {
                choice = (choice + 1) % SCENES.len();
            }
            self.force(SCENES[choice].id, world, player, forward);
            self.next = rng.range(24., 32.);
        }
        if self.remaining == 0. {
            self.active.fill(false);
            self.crouch = false;
            return;
        }
        let Some(index) = self.scene else {
            return;
        };
        let scene = SCENES[index];
        if self.started {
            self.started = false;
            if scene.id == 5 {
                self.lift_player = true;
                for (i, c) in cars.iter().take(BRIDGE_CAPACITY).enumerate() {
                    physics.launch(i, c.pos, Vec3::new(0., 6., 0.), 8.);
                }
                for (i, n) in npcs.iter().take(BRIDGE_CAPACITY).enumerate() {
                    physics.npc_airborne[i] = true;
                    physics.npc_position[i] = n.pos;
                    physics.npc_velocity[i] = Vec3::new(0., 4., 0.);
                }
            }
        }
        self.crouch = scene.id == 80 && self.elapsed > 2.;
        if scene.id == 10 {
            self.crouch = dot(direction(sub(self.position[0], eye)), forward) > 0.85
                && self.position[0].distance(player) < 80.;
        }
        if scene.id == 44 {
            self.swap -= dt;
            if self.swap <= 0. {
                self.swap += 5.;
                let n = npcs.len();
                if n > 0 {
                    let i = rng.below(n as u32) as usize;
                    std::mem::swap(&mut self.position[0], &mut npcs[i].pos);
                }
            }
        }
        if scene.id == 94 {
            self.printer += dt * 30.;
            while self.printer >= 1. {
                self.printer -= 1.;
                let i = self.pigeon_cursor;
                self.pigeon_cursor = PIGEONS.start + (i + 1 - PIGEONS.start) % PIGEONS.len();
                let p = Vec3::new(
                    self.position[0].x,
                    self.position[0].y + 1.,
                    self.position[0].z + 1.,
                );
                self.activate(i, Model::Pigeon, p, 0.35);
                self.velocity[i] =
                    Vec3::new(rng.range(-8., 8.), rng.range(2., 6.), rng.range(3., 10.));
            }
        }
        for i in 0..ACTORS {
            if !self.active[i] {
                continue;
            }
            if scene.id == 94 && i >= PIGEONS.start {
                let v = self.velocity[i];
                self.position[i].x += v.x * dt;
                self.position[i].y += v.y * dt;
                self.position[i].z += v.z * dt;
                if self.position[i].distance(self.position[0]) > 45. {
                    self.active[i] = false;
                }
                continue;
            }
            if self.model[i] == Model::Cucumber {
                let a = self.elapsed + i as f32;
                self.position[i] = Vec3::new(
                    self.center.x + a.cos() * 2.,
                    self.center.y + 1. + (a * 2.).sin() * 0.5,
                    self.center.z + a.sin() * 2.,
                );
                self.yaw[i] = a;
                continue;
            }
            let motion = if scene.id == 24 && self.model[i] == Model::Triangle {
                Motion::Escape
            } else if scene.id == 89 && i > 0 {
                Motion::Orbit
            } else {
                scene.motion
            };
            match motion {
                Motion::Rain | Motion::Fall => {
                    self.velocity[i].y -= 18. * dt;
                    self.position[i].y += self.velocity[i].y * dt;
                    let floor =
                        world.surface(self.position[i].x, self.position[i].z) + self.scale[i];
                    if self.position[i].y < floor {
                        if motion == Motion::Rain {
                            self.position[i].y = floor + rng.range(25., 60.);
                            self.velocity[i].y = 0.;
                        } else {
                            self.position[i].y = floor;
                            self.velocity[i].y = 0.;
                        }
                    }
                    if motion == Motion::Rain {
                        self.yaw[i] = (self.yaw[i] + dt * 2.).rem_euclid(std::f32::consts::TAU);
                    }
                }
                Motion::Chase | Motion::Escape | Motion::Walk => {
                    let target = if scene.id == 24 && self.model[i] == Model::Square {
                        self.position[3 + i % 4]
                    } else if motion == Motion::Walk {
                        Vec3::new(
                            self.center.x + (self.elapsed * 0.3 + i as f32).cos() * 14.,
                            0.,
                            self.center.z + (self.elapsed * 0.3 + i as f32).sin() * 14.,
                        )
                    } else {
                        player
                    };
                    let mut d = sub(target, self.position[i]);
                    d.y = 0.;
                    d = direction(d);
                    let speed = if scene.id == 1 {
                        50.
                    } else if scene.id == 36 {
                        0.8
                    } else if motion == Motion::Walk {
                        2.
                    } else {
                        6.
                    };
                    let sign = if motion == Motion::Escape { -1. } else { 1. };
                    if !(scene.id == 52 && (self.elapsed % 4.) > 2.) {
                        self.position[i].x += d.x * speed * dt * sign;
                        self.position[i].z += d.z * speed * dt * sign;
                    }
                    self.position[i].y =
                        world.surface(self.position[i].x, self.position[i].z) + self.scale[i];
                    self.yaw[i] = if scene.id == 1 {
                        (-d.x).atan2(-d.z)
                    } else {
                        d.x.atan2(d.z)
                    };
                    if scene.id == 1 {
                        for (c_id, c) in cars.iter().take(BRIDGE_CAPACITY).enumerate() {
                            if c.pos.distance(self.position[i]) < 3.
                                && (c.pos.y - self.position[i].y).abs() < 4.
                            {
                                physics.launch(c_id, c.pos, Vec3::new(0., 999., 0.), 1.);
                            }
                        }
                    }
                }
                Motion::Orbit | Motion::Fly => {
                    let rate = if scene.id == 8 { 83.333 / 14. } else { 0.4 };
                    let a = self.elapsed * rate + i as f32 * 1.7;
                    let radius = if scene.count > 1 { 8. + i as f32 } else { 14. };
                    if scene.id == 97 || scene.id == 7 {
                        let phase = player.z.atan2(player.x) + self.elapsed * 0.25;
                        self.position[i] = Vec3::new(
                            phase.cos() * self.orbit_radius,
                            world.surface(player.x, player.z) + 18.,
                            phase.sin() * self.orbit_radius,
                        );
                    } else if !(scene.id == 48 && world.water_depth(self.position[i]) > 0.1) {
                        self.position[i].x = self.center.x + a.cos() * radius;
                        self.position[i].z = self.center.z + a.sin() * radius;
                        self.position[i].y = world.surface(self.position[i].x, self.position[i].z)
                            + self.scale[i]
                            + if motion == Motion::Fly {
                                8. + (a * 2.).sin() * 4.
                            } else {
                                0.
                            };
                    }
                    self.yaw[i] = a + std::f32::consts::FRAC_PI_2;
                    if scene.id == 100 {
                        self.position[i].y = (90. - self.elapsed * 8.).max(self.center.y + 30.);
                    }
                }
                Motion::Still => {}
            }
            if scene.id == 2
                && self.elapsed > 5.
                && npcs.iter().any(|n| n.pos.distance(self.position[i]) < 15.)
            {
                self.model[i] = Model::Wardrobe;
            }
            if scene.id == 74 {
                self.position[i].y = self.center.y + (self.elapsed * 2.).sin() * 10.;
            }
            if scene.id == 96 {
                self.position[i].y =
                    self.center.y + self.scale[i] + (self.elapsed * 2.).sin().abs() * 25.;
            }
            self.position[i].x = finite(self.position[i].x, 2000.);
            self.position[i].y = finite(self.position[i].y, 1000.);
            self.position[i].z = finite(self.position[i].z, 2000.);
        }
    }
    pub fn compensate_gravity(&self, physics: &mut PhysicsSaboteur, dt: f32) {
        if self.gravity_off() {
            for i in 0..BRIDGE_CAPACITY {
                if physics.car_airborne[i] {
                    physics.car_velocity[i].y += 18. * dt;
                }
                if physics.npc_airborne[i] {
                    physics.npc_velocity[i].y += 18. * dt;
                }
            }
        }
    }
    pub fn interact(&self, p: Vec3) -> Option<Interaction> {
        if self.remaining <= 0. || !self.active[0] || self.position[0].distance(p) > 8. {
            return None;
        }
        match self.source_id() {
            9 => Some(Interaction::BuyDumplings),
            26 => Some(Interaction::Message(
                "ЦЕ КАРТОННА RTX. У САПЕРІ СТАБІЛЬНІ 9000 FPS.",
            )),
            37 => Some(Interaction::Message(
                "SKILL ISSUE. БАНКОМАТ ВИДАВ ПАПІРЕЦЬ.",
            )),
            58 => Some(Interaction::Rest),
            82 => Some(Interaction::Portal),
            _ => None,
        }
    }
    pub fn office_party(&mut self, p: Vec3) {
        self.remaining = 20.;
        self.next = 26.;
        self.elapsed = 0.;
        self.center = p;
        self.active.fill(false);
        self.position[0] = p;
        for i in 1..13 {
            self.activate(i, Model::Cucumber, Vec3::new(p.x, p.y + 1., p.z), 0.8);
        }
    }
    pub fn render(&self, r: &mut Renderer, eye: Vec3) {
        r.set_chaos_crouch(false);
        for i in 0..ACTORS {
            if !self.active[i] {
                continue;
            }
            let p = self.position[i];
            let scale = self.scale[i];
            if p.distance(eye) > 280. + scale * 2. {
                continue;
            }
            if let Some(mesh) = self.meshes[self.model[i] as usize] {
                let color = if self.model[i] == Model::Jpeg {
                    const C: [[f32; 3]; 4] =
                        [[1., 0., 1.], [0., 1., 1.], [1., 1., 0.], [0.1, 0.9, 0.2]];
                    C[i % 4]
                } else {
                    [1.; 3]
                };
                r.chaos_model(
                    mesh,
                    [p.x, p.y, p.z],
                    self.yaw[i],
                    scale,
                    color,
                    None,
                    0.,
                    None,
                );
            }
            if matches!(self.model[i], Model::Titan | Model::Granny) {
                if let Some(rotor) = self.meshes[if self.model[i] == Model::Granny {
                    Model::Umbrella as usize
                } else {
                    Model::Rotor as usize
                }] {
                    r.chaos_model(
                        rotor,
                        [
                            p.x,
                            p.y + scale
                                * if self.model[i] == Model::Granny {
                                    1.4
                                } else {
                                    1.08
                                },
                            p.z,
                        ],
                        self.clock * 40.,
                        scale,
                        [1.; 3],
                        None,
                        0.,
                        None,
                    );
                }
            }
        }
        if self.remaining > 0.
            && (self.source_id() == 45
                || self.source_id() == 71 && self.elapsed > 14.
                || self.source_id() == 100 && self.elapsed > 5.)
        {
            if let Some(mesh) = self.meshes[Model::Jpeg as usize] {
                for i in 0..24 {
                    let a = i as f32 * 2.399963;
                    let t = (self.elapsed + i as f32 * 0.13) % 2.;
                    let p = self.position[0];
                    let color = if i % 2 == 0 {
                        [1., 0., 0.7]
                    } else {
                        [0., 1., 0.8]
                    };
                    r.chaos_model(
                        mesh,
                        [
                            p.x + a.cos() * t * 4.,
                            p.y + 5. + t * 6. - t * t * 4.,
                            p.z + a.sin() * t * 4.,
                        ],
                        a + self.elapsed,
                        0.15,
                        color,
                        None,
                        0.,
                        None,
                    );
                }
            }
        }
    }
    pub fn render_ui(&self, r: &mut Renderer, eye: Vec3, yaw: f32, pitch: f32) {
        if self.remaining <= 0. {
            return;
        }
        let Some(index) = self.scene else {
            return;
        };
        let scene = SCENES[index];
        r.panel([175., 205., 675., 61.], [0.04, 0.02, 0.08], 0.85, 6.);
        r.label(190., 211., 1.55, scene.title, [1., 0.3, 0.85]);
        r.label(720., 213., 0.9, "F6: ДАЛІ", [0.8, 0.8, 1.]);
        r.label(190., 238., 1.0, scene.caption, [1., 0.95, 0.4]);
        if self.source_id() == 100 && self.elapsed > 5. {
            r.panel([185., 285., 660., 110.], [0., 0.05, 0.05], 0.95, 4.);
            r.label(
                198.,
                298.,
                1.3,
                "WINDOWS 98 / ADMIN CONSOLE",
                [0.2, 1., 0.3],
            );
            r.label(
                198.,
                330.,
                1.3,
                "> sudo apt install cucumber",
                [0.2, 1., 0.3],
            );
            r.label(198., 365., 1.3, "404: MONDAY NOT FOUND", [1., 0.2, 0.2]);
        }
        if self.source_id() == 71 && self.elapsed > 14. {
            r.label(350., 285., 2., "ЗНОВУ НОВИЙ РІК!", [0., 1., 0.5]);
        }
        if self.source_id() == 82 && !self.active[0] {
            r.label(320., 285., 1.4, "ОФІС ЛІТАЮЧИХ ОГІРКІВ", [0., 1., 0.3]);
        }
        if self.source_id() == 51 {
            r.panel([640., 285., 250., 110.], [0.15, 0.15, 0.2], 0.95, 4.);
            r.label(653., 295., 1.1, "OPEN CITY.EXE", [1.; 3]);
            r.label(653., 325., 1.1, "DELETE MONDAY", [1.; 3]);
            r.label(653., 355., 1.1, "CTRL+Z UNIVERSE", [1.; 3]);
        }
        let p = self.position[0];
        let dx = p.x - eye.x;
        let dy = p.y + self.scale[0] - eye.y;
        let dz = p.z - eye.z;
        let right = dx * yaw.cos() + dz * yaw.sin();
        let horizontal = dx * yaw.sin() - dz * yaw.cos();
        let depth = horizontal * pitch.cos() - dy * pitch.sin();
        let up = dy * pitch.cos() + horizontal * pitch.sin();
        if depth > 1. {
            let aspect = r.width as f32 / r.height.max(1) as f32;
            let x = 512. + right / (depth * 0.7 * aspect) * 512.;
            let y = 288. - up / (depth * 0.7) * 288.;
            if (0. ..900.).contains(&x) && (70. ..500.).contains(&y) {
                r.label(
                    x,
                    y,
                    0.9,
                    if scene.id == 2 && self.model[0] == Model::Wardrobe {
                        "ТЕПЕР Я ШАФА"
                    } else {
                        scene.caption
                    },
                    [1., 1., 0.2],
                );
            }
        }
    }
}

fn potato_pixels() -> [u8; 16 * 16 * 3] {
    let mut pixels = [210; 16 * 16 * 3];
    for y in 0..16 {
        for x in 0..16 {
            let dx = x as i32 - 8;
            let dy = y as i32 - 8;
            let color = if dx * dx * 2 + dy * dy < 65 {
                [184, 139, 73]
            } else {
                [230, 0, 210]
            };
            let i = (y * 16 + x) * 3;
            pixels[i..i + 3].copy_from_slice(&color);
            if ((y == 6) && (x == 5 || x == 10)) || ((8..=10).contains(&y) && (3..=12).contains(&x))
            {
                pixels[i..i + 3].copy_from_slice(&[25, 15, 10]);
            }
        }
    }
    pixels
}

fn box_part(m: &mut Mesh, p: [f32; 3], s: [f32; 3], c: [f32; 3]) {
    m.cuboid(p, s, c, 0, 0.);
}
fn ball(m: &mut Mesh, p: [f32; 3], s: [f32; 3], c: [f32; 3]) {
    for row in 0..5 {
        for col in 0..10 {
            let point = |r: usize, k: usize| {
                let a = std::f32::consts::PI * r as f32 / 5.;
                let b = std::f32::consts::TAU * k as f32 / 10.;
                [
                    p[0] + s[0] * a.sin() * b.cos(),
                    p[1] + s[1] * a.cos(),
                    p[2] + s[2] * a.sin() * b.sin(),
                ]
            };
            m.quad(
                [
                    point(row, col),
                    point(row + 1, col),
                    point(row + 1, col + 1),
                    point(row, col + 1),
                ],
                c,
                0,
                0.,
            );
        }
    }
}
fn human(m: &mut Mesh, shirt: [f32; 3], skin: [f32; 3]) {
    box_part(m, [0., 0., 0.], [0.7, 0.8, 0.4], shirt);
    box_part(m, [0., 0.7, 0.], [0.45, 0.45, 0.45], skin);
    for side in [-1., 1.] {
        box_part(m, [side * 0.65, 0.2, 0.], [0.65, 0.25, 0.3], skin);
        box_part(
            m,
            [side * 0.2, -0.65, 0.],
            [0.28, 0.7, 0.3],
            [0.18, 0.2, 0.24],
        );
    }
}
fn fridge(m: &mut Mesh, police: bool) {
    box_part(m, [0., 0., 0.], [1.3, 2., 0.9], [0.84, 0.9, 0.9]);
    box_part(m, [0., 0.1, 0.48], [1.05, 1.65, 0.04], [0.04, 0.05, 0.08]);
    box_part(m, [0.8, 0., 0.7], [0.12, 2., 0.9], [0.8, 0.88, 0.9]);
    // A tiny policeman and a genuinely alternating chessboard inside the open fridge.
    box_part(m, [0., 0.05, 0.59], [0.35, 0.45, 0.24], [0.05, 0.15, 0.7]);
    box_part(m, [0., 0.4, 0.6], [0.24, 0.24, 0.2], [0.7, 0.5, 0.35]);
    for x in 0..4 {
        for z in 0..4 {
            box_part(
                m,
                [-0.3 + x as f32 * 0.2, -0.35, 0.35 + z as f32 * 0.15],
                [0.19, 0.03, 0.14],
                if (x + z) % 2 == 0 {
                    [0.95; 3]
                } else {
                    [0.02; 3]
                },
            );
        }
    }
    if police {
        ball(m, [0., -1.1, 0.], [0.3, 0.4, 0.25], [0.04; 3]);
        box_part(m, [0., 1.15, 0.], [0.6, 0.2, 0.3], [0., 0.2, 1.]);
    }
}

fn build_model(model: Model) -> Mesh {
    if matches!(
        model,
        Model::ToiletArmy | Model::GrannyArmy | Model::WasherParade | Model::Bodyguards
    ) {
        let (base, count, columns, spacing) = match model {
            Model::ToiletArmy => (Model::Toilet, 400, 20, 2.5),
            Model::GrannyArmy => (Model::Granny, 300, 20, 2.2),
            Model::WasherParade => (Model::Washer, 90, 10, 2.5),
            _ => (Model::Titan, 12, 4, 3.),
        };
        let template = formation_template(base);
        let mut mesh = Mesh {
            vertices: Vec::with_capacity(template.vertices.len() * count + 300),
        };
        for i in 0..count {
            let offset = [
                (i % columns) as f32 * spacing - columns as f32 * spacing / 2.,
                0.,
                (i / columns) as f32 * spacing,
            ];
            for v in &template.vertices {
                let mut v = *v;
                for (axis, offset) in offset.into_iter().enumerate() {
                    v.position[axis] += offset;
                }
                mesh.vertices.push(v);
            }
        }
        if model == Model::Bodyguards {
            ball(&mut mesh, [0., -0.3, -3.], [0.5, 0.5, 0.5], [0.2, 0.8, 0.1]);
        }
        return mesh;
    }
    let mut m = Mesh {
        vertices: Vec::with_capacity(4096),
    };
    match model {
        Model::Titan => {
            human(&mut m, [1., 0.02, 0.6], [0.005; 3]);
            for side in [-1., 1.] {
                for (i, c) in [[1., 0., 0.], [1., 0.7, 0.], [0., 1., 0.], [0., 0.3, 1.]]
                    .into_iter()
                    .enumerate()
                {
                    box_part(
                        &mut m,
                        [side * 0.2, -0.4 - i as f32 * 0.16, 0.],
                        [0.3, 0.15, 0.34],
                        c,
                    );
                }
            }
            box_part(&mut m, [0., 1., 0.], [0.7, 0.1, 0.6], [1., 0., 0.5]);
        }
        Model::Dumpling | Model::Admin => {
            ball(&mut m, [0., 0.2, 0.], [1., 0.65, 0.65], [0.9, 0.79, 0.57]);
            for side in [-1., 1.] {
                box_part(&mut m, [side * 0.45, -0.65, 0.], [0.25, 0.7, 0.3], [0.2; 3]);
                box_part(&mut m, [side * 0.3, 0.35, 0.6], [0.1, 0.1, 0.06], [0.01; 3]);
            }
            if model == Model::Admin {
                box_part(&mut m, [0., 0.15, 0.68], [1.1, 0.45, 0.05], [0.1, 0.2, 0.8]);
                for (i, c) in [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.], [1., 1., 0.]]
                    .into_iter()
                    .enumerate()
                {
                    box_part(
                        &mut m,
                        [
                            -0.3 + (i % 2) as f32 * 0.5,
                            0.2 + (i / 2) as f32 * 0.2,
                            0.73,
                        ],
                        [0.3, 0.15, 0.02],
                        c,
                    );
                }
            }
        }
        Model::Wardrobe => {
            for (i, c) in [
                [1., 0., 0.],
                [1., 0.6, 0.],
                [0., 1., 0.],
                [0., 0.3, 1.],
                [0.6, 0., 1.],
            ]
            .into_iter()
            .enumerate()
            {
                box_part(
                    &mut m,
                    [-0.64 + i as f32 * 0.32, 0., 0.],
                    [0.31, 2., 0.7],
                    c,
                );
            }
            for side in [-1., 1.] {
                box_part(&mut m, [side * 1., 0.1, 0.], [0.6, 0.3, 0.3], [0.02; 3]);
                box_part(&mut m, [side * 0.5, -1.1, 0.], [0.2, 0.3, 0.3], [0.1; 3]);
            }
        }
        Model::Fridge | Model::PoliceFridge => fridge(&mut m, model == Model::PoliceFridge),
        Model::Banana => {
            for i in 0..7 {
                let a = i as f32 * 0.24;
                ball(
                    &mut m,
                    [a.sin() * 0.8, a.cos() * 1.5 - 0.7, 0.],
                    [0.25, 0.3, 0.25],
                    [1., 0.85, 0.02],
                );
            }
            box_part(&mut m, [0., 1., 0.], [0.9, 0.25, 0.6], [0.9; 3]);
            box_part(
                &mut m,
                [0.6, 0.1, 0.7],
                [0.6, 0.16, 0.16],
                [0.35, 0.15, 0.04],
            );
        }
        Model::Borscht => {
            ball(&mut m, [0., 0., 0.], [1.2, 0.65, 1.], [0.4; 3]);
            ball(&mut m, [0., 0.4, 0.], [1.05, 0.13, 0.85], [0.7, 0.01, 0.03]);
            for side in [-1., 1.] {
                box_part(&mut m, [side * 1.3, 0.1, 0.], [0.5, 0.15, 0.2], [0.2; 3]);
            }
        }
        Model::Toilet | Model::Portal => {
            box_part(&mut m, [0., 0.35, -0.4], [0.8, 0.9, 0.35], [0.9; 3]);
            ball(&mut m, [0., -0.15, 0.2], [0.6, 0.3, 0.7], [0.95; 3]);
            ball(
                &mut m,
                [0., 0.05, 0.3],
                [0.35, 0.05, 0.45],
                if model == Model::Portal {
                    [0.8, 0., 1.]
                } else {
                    [0.1, 0.2, 0.4]
                },
            );
            for side in [-1., 1.] {
                box_part(&mut m, [side * 0.3, -0.7, 0.], [0.17, 0.6, 0.2], [0.8; 3]);
            }
        }
        Model::Chair => {
            human(&mut m, [0.3, 0.2, 0.5], [0.7, 0.5, 0.4]);
            box_part(&mut m, [0., -0.45, 0.], [1., 0.2, 0.9], [0.08; 3]);
            box_part(&mut m, [0., 0., -0.5], [1., 1.2, 0.2], [0.08; 3]);
            box_part(&mut m, [0., -1., 0.], [1.2, 0.1, 1.2], [0.15; 3]);
        }
        Model::Cow => {
            box_part(&mut m, [0., 0.25, 0.], [2.4, 1., 1.], [0.88; 3]);
            box_part(&mut m, [1.4, 0.5, 0.], [0.7, 0.65, 0.7], [0.15; 3]);
            for x in 0..8 {
                for side in [-1., 1.] {
                    box_part(
                        &mut m,
                        [-1.05 + x as f32 * 0.3, -0.6, side * 0.45],
                        [0.12, 0.9, 0.12],
                        if x % 2 == 0 { [0.02; 3] } else { [0.9; 3] },
                    );
                }
            }
        }
        Model::Granny => {
            human(&mut m, [0.35, 0.25, 0.5], [0.7, 0.5, 0.4]);
            ball(&mut m, [0., 1., -0.1], [0.3, 0.25, 0.3], [0.85; 3]);
            box_part(&mut m, [0., -0.25, 0.], [0.9, 0.8, 0.6], [0.4, 0.2, 0.5]);
        }
        Model::Umbrella => {
            let corners = [[-1., 0., -1.], [1., 0., -1.], [1., 0., 1.], [-1., 0., 1.]];
            for i in 0..4 {
                m.triangle(
                    [[0., 0.4, 0.], corners[i], corners[(i + 1) % 4]],
                    if i % 2 == 0 {
                        [1., 0., 0.6]
                    } else {
                        [0., 0.8, 1.]
                    },
                    0,
                    0.,
                );
            }
            box_part(&mut m, [0., -0.3, 0.], [0.06, 0.6, 0.06], [0.2; 3]);
        }
        Model::Rotor => {
            box_part(&mut m, [0., 0., 0.], [2.4, 0.08, 0.18], [1., 0.02, 0.6]);
            box_part(&mut m, [0., 0., 0.], [0.18, 0.08, 2.4], [0., 0.8, 1.]);
        }
        Model::Sign => {
            box_part(&mut m, [0., 0., 0.], [0.12, 2., 0.12], [0.5; 3]);
            box_part(&mut m, [0., 0.8, 0.], [1.5, 0.6, 0.08], [1., 0.1, 0.7]);
            ball(&mut m, [0., 0.8, 0.1], [0.45, 0.2, 0.1], [0.95, 0.85, 0.65]);
        }
        Model::Tv => {
            box_part(&mut m, [0., 0., 0.], [2.5, 1.7, 0.7], [0.3; 3]);
            box_part(&mut m, [0., 0., 0.4], [2.1, 1.35, 0.08], [0.01, 0.02, 0.03]);
            for i in 0..5 {
                let k = 0.6_f32.powi(i);
                box_part(
                    &mut m,
                    [0., 0., 0.47 + i as f32 * 0.02],
                    [1.7 * k, 1.05 * k, 0.02],
                    if i % 2 == 0 {
                        [0.1, 0.8, 0.3]
                    } else {
                        [0.01; 3]
                    },
                );
            }
        }
        Model::Centaur => {
            box_part(&mut m, [0., -0.4, 0.], [1., 1., 1.], [0.9; 3]);
            ball(
                &mut m,
                [0., -0.4, 0.55],
                [0.35, 0.35, 0.06],
                [0.05, 0.15, 0.25],
            );
            human(&mut m, [1., 0., 0.5], [0.02; 3]);
        }
        Model::ErrorHead | Model::Ping => {
            human(&mut m, [0.15, 0.2, 0.6], [0.7, 0.5, 0.4]);
            box_part(&mut m, [0., 0.8, 0.], [0.9, 0.65, 0.7], [0.65; 3]);
            box_part(
                &mut m,
                [0., 0.8, 0.38],
                [0.75, 0.5, 0.02],
                if model == Model::Ping {
                    [0., 0.7, 0.1]
                } else {
                    [0., 0.05, 0.8]
                },
            );
        }
        Model::Sausage | Model::Bread => {
            ball(
                &mut m,
                [0., 0., 0.],
                [1., 0.28, 0.28],
                if model == Model::Sausage {
                    [0.7, 0.15, 0.05]
                } else {
                    [0.8, 0.55, 0.22]
                },
            );
            for i in 0..5 {
                box_part(
                    &mut m,
                    [-0.7 + i as f32 * 0.35, 0.25, 0.],
                    [0.08, 0.06, 0.3],
                    [0.95, 0.8, 0.5],
                );
            }
        }
        Model::Square => {
            box_part(&mut m, [0., 0., 0.], [1.5, 1.5, 0.15], [0.002; 3]);
            for side in [-1., 1.] {
                box_part(&mut m, [side, 0., 0.], [0.7, 0.4, 0.4], [0.01; 3]);
            }
        }
        Model::Triangle => {
            m.triangle(
                [[-0.6, -0.5, 0.], [0.6, -0.5, 0.], [0., 0.7, 0.]],
                [1., 0., 0.7],
                0,
                0.,
            );
            m.triangle(
                [[0., 0.7, 0.], [0.6, -0.5, 0.], [-0.6, -0.5, 0.]],
                [1., 0., 0.7],
                0,
                0.,
            );
        }
        Model::Cucumber => {
            ball(&mut m, [0., 0., 0.], [0.3, 1., 0.3], [0.1, 0.65, 0.08]);
            for side in [-1., 1.] {
                box_part(
                    &mut m,
                    [side * 0.13, 0.4, 0.3],
                    [0.06, 0.08, 0.03],
                    [0.01; 3],
                );
            }
        }
        Model::Grandpa => {
            human(&mut m, [0.3; 3], [0.7, 0.5, 0.4]);
            for i in 0..4 {
                box_part(
                    &mut m,
                    [-0.75 + i as f32 * 0.5, 0.1, 0.5],
                    [0.45, 0.55, 0.05],
                    [0.1, 0.8, 0.05],
                );
            }
        }
        Model::Atm => {
            box_part(&mut m, [0., 0., 0.], [1.2, 2., 0.7], [0.3, 0.4, 0.5]);
            box_part(&mut m, [0., 0.4, 0.4], [0.85, 0.6, 0.06], [0., 0.05, 0.2]);
            box_part(&mut m, [0., -0.1, 0.48], [0.65, 0.08, 0.4], [0.95; 3]);
        }
        Model::Kettle => {
            human(&mut m, [1., 0., 0.7], [0.02; 3]);
            ball(&mut m, [0., 1., 0.], [0.7, 0.6, 0.6], [0.6; 3]);
            box_part(&mut m, [0.8, 1.1, 0.], [0.8, 0.2, 0.25], [0.6; 3]);
        }
        Model::Bus => {
            box_part(&mut m, [0., 0., 0.], [1.5, 1.5, 3.5], [0.85, 0.7, 0.02]);
            for z in 0..5 {
                box_part(
                    &mut m,
                    [0.76, 0.35, -1.3 + z as f32 * 0.6],
                    [0.03, 0.5, 0.4],
                    [0.1, 0.3, 0.4],
                );
            }
            for x in [-0.8, 0.8] {
                for z in [-1., 1.] {
                    ball(&mut m, [x, -0.65, z], [0.15, 0.35, 0.35], [0.02; 3]);
                }
            }
        }
        Model::Cursor => {
            m.triangle(
                [[0., 1., 0.], [0., -1., 0.], [0.65, -0.55, 0.]],
                [1.; 3],
                0,
                0.,
            );
            m.quad(
                [
                    [0.2, -0.4, 0.],
                    [0.45, -1., 0.],
                    [0.7, -0.9, 0.],
                    [0.45, -0.3, 0.],
                ],
                [1.; 3],
                0,
                0.,
            );
        }
        Model::Cat => {
            ball(&mut m, [0., -0.3, 0.], [0.7, 0.5, 0.5], [0.9, 0.5, 0.1]);
            box_part(&mut m, [0., 0.5, 0.], [0.9, 0.8, 0.8], [0.3; 3]);
            for side in [-1., 1.] {
                m.triangle(
                    [
                        [side * 0.2, 0.8, 0.],
                        [side * 0.5, 1.2, 0.],
                        [side * 0.5, 0.8, 0.],
                    ],
                    [0.9, 0.5, 0.1],
                    0,
                    0.,
                );
            }
        }
        Model::Washer => {
            box_part(&mut m, [0., 0., 0.], [1.4, 1.8, 1.], [0.9; 3]);
            ball(&mut m, [0., -0.1, 0.57], [0.5, 0.5, 0.05], [0.1, 0.3, 0.4]);
            box_part(&mut m, [0.35, 0.6, 0.55], [0.2, 0.12, 0.04], [1., 0., 0.5]);
        }
        Model::Pigeon | Model::Duck => {
            ball(&mut m, [0., 0., 0.], [0.5, 0.4, 0.7], [0.7; 3]);
            ball(&mut m, [0., 0.45, 0.35], [0.25, 0.25, 0.25], [0.3; 3]);
            box_part(&mut m, [0., 0.4, 0.7], [0.16, 0.12, 0.25], [1., 0.7, 0.]);
            for side in [-1., 1.] {
                box_part(&mut m, [side * 0.65, 0.1, 0.], [1., 0.08, 0.6], [0.8; 3]);
            }
            if model == Model::Duck {
                box_part(&mut m, [0., -0.25, 0.8], [0.8, 0.45, 0.04], [0.95; 3]);
            }
        }
        Model::Sofa => {
            box_part(&mut m, [0., -0.25, 0.], [2., 0.5, 1.], [0.3, 0.1, 0.5]);
            box_part(&mut m, [0., 0.3, -0.5], [2., 1., 0.2], [0.4, 0.1, 0.6]);
            for side in [-1., 1.] {
                box_part(&mut m, [side * 0.9, -0.75, 0.], [0.5, 0.45, 1.5], [0.05; 3]);
            }
        }
        Model::Note => {
            box_part(&mut m, [0.35, 0.2, 0.], [0.15, 1.5, 0.15], [1., 0., 0.8]);
            ball(&mut m, [0., -0.5, 0.], [0.5, 0.25, 0.2], [1., 0., 0.8]);
        }
        Model::Watermelon | Model::Cabbage | Model::Globe => {
            ball(
                &mut m,
                [0., 0., 0.],
                [0.9, 0.8, 0.8],
                if model == Model::Globe {
                    [0.1, 0.4, 0.95]
                } else {
                    [0.15, 0.65, 0.08]
                },
            );
            for i in 0..5 {
                box_part(
                    &mut m,
                    [-0.6 + i as f32 * 0.3, 0., 0.7],
                    [0.1, 1., 0.1],
                    [0.05, 0.3, 0.05],
                );
            }
            if model == Model::Globe {
                for side in [-1., 1.] {
                    box_part(
                        &mut m,
                        [side * 0.4, -1., 0.],
                        [0.2, 0.6, 0.2],
                        [1., 1., 0.8],
                    );
                }
            }
        }
        Model::Clock => {
            ball(&mut m, [0., 0., 0.], [1., 1., 0.15], [0.95; 3]);
            for i in 0..73 {
                let a = i as f32 * 2.399_963;
                let end = [a.cos() * 0.85, a.sin() * 0.85, 0.2];
                m.quad(
                    [
                        [0., 0., 0.2],
                        [0.03, 0.03, 0.2],
                        end,
                        [end[0] + 0.02, end[1] + 0.02, 0.2],
                    ],
                    [0.1, 0.05, 0.1],
                    0,
                    0.,
                );
            }
        }
        Model::Brick => {
            box_part(&mut m, [0., 0., 0.], [1.4, 0.65, 0.6], [0.7, 0.22, 0.1]);
            box_part(&mut m, [0., 0.5, 0.], [0.9, 0.1, 0.9], [0.02; 3]);
            box_part(&mut m, [0., -0.3, 0.4], [0.6, 0.5, 0.08], [0.95; 3]);
        }
        Model::Worm => {
            for i in 0..7 {
                ball(
                    &mut m,
                    [0., i as f32 * 0.3 - 1., (i as f32 * 0.6).sin() * 0.3],
                    [0.5, 0.35, 0.5],
                    [1., 0., 0.6],
                );
            }
            for side in [-1., 1.] {
                box_part(
                    &mut m,
                    [side * 0.7, 0.5, 0.],
                    [0.7, 0.4, 0.4],
                    [1., 0., 0.6],
                );
            }
        }
        Model::Jpeg => box_part(&mut m, [0., 0., 0.], [1.5, 0.15, 1.5], [1.; 3]),
        Model::FKey => {
            box_part(&mut m, [0., 0., 0.], [2., 1., 2.], [0.2; 3]);
            for (row, bits) in [7, 4, 6, 4, 4].into_iter().enumerate() {
                for col in 0..3 {
                    if bits & (1 << col) != 0 {
                        box_part(
                            &mut m,
                            [-0.45 + col as f32 * 0.3, 0.55, -0.6 + row as f32 * 0.3],
                            [0.25, 0.08, 0.25],
                            [1.; 3],
                        );
                    }
                }
            }
        }
        Model::Generator => {
            box_part(&mut m, [0., -0.6, 0.], [1., 0.6, 0.6], [0.2; 3]);
            for side in [-1., 1.] {
                ball(&mut m, [side * 0.7, 0., 0.], [0.4, 0.5, 0.4], [0.95; 3]);
                box_part(&mut m, [side * 0.7, 0.6, 0.], [0.2, 0.3, 0.2], [1., 0., 0.]);
            }
            box_part(&mut m, [0., 0.7, 0.], [0.08, 1.3, 0.08], [0.5; 3]);
            ball(&mut m, [0., 1.4, 0.], [0.25, 0.3, 0.25], [1., 1., 0.4]);
        }
        Model::King | Model::Toaster => {
            box_part(&mut m, [0., 0., 0.], [1.2, 0.8, 0.8], [0.8; 3]);
            box_part(&mut m, [0., 0., 0.45], [0.8, 0.5, 0.03], [0.03; 3]);
            if model == Model::King {
                box_part(&mut m, [0., -0.8, 0.], [1.8, 0.4, 1.5], [1., 0.7, 0.]);
                box_part(&mut m, [0., 0.8, -0.6], [1.8, 2., 0.3], [1., 0.7, 0.]);
            } else {
                box_part(&mut m, [0., 0.5, 0.], [0.8, 0.3, 0.2], [0.8, 0.5, 0.15]);
            }
        }
        Model::Crosswalk => {
            for i in 0..9 {
                box_part(
                    &mut m,
                    [-1.6 + i as f32 * 0.4, -0.98, 0.],
                    [0.22, 0.03, 2.5],
                    [0.95; 3],
                );
            }
        }
        Model::Printer => {
            box_part(&mut m, [0., 0., 0.], [1.5, 0.7, 1.], [0.85; 3]);
            box_part(&mut m, [0., 0.5, -0.4], [1., 0.7, 0.05], [0.95; 3]);
            box_part(&mut m, [0., -0.1, 0.7], [1., 0.1, 0.7], [0.15; 3]);
        }
        Model::Mushroom => {
            box_part(&mut m, [0., -0.4, 0.], [0.6, 1.2, 0.6], [0.85, 0.75, 0.5]);
            for x in -3i32..=3 {
                for z in -3i32..=3 {
                    if x * x + z * z <= 10 {
                        box_part(
                            &mut m,
                            [x as f32 * 0.3, 0.4, z as f32 * 0.3],
                            [0.3, 0.4, 0.3],
                            if (x + z) % 3 == 0 {
                                [1.; 3]
                            } else {
                                [1., 0., 0.2]
                            },
                        );
                    }
                }
            }
        }
        Model::Foot => {
            box_part(&mut m, [0., 0.3, 0.], [0.6, 1.5, 0.6], [0.9, 0.55, 0.35]);
            ball(&mut m, [0., -0.7, 0.4], [0.5, 0.3, 0.8], [0.9, 0.55, 0.35]);
            for i in 0..5 {
                ball(
                    &mut m,
                    [-0.4 + i as f32 * 0.2, -0.65, 1.],
                    [0.13, 0.2, 0.2],
                    [0.9, 0.55, 0.35],
                );
            }
        }
        Model::Pine => {
            box_part(&mut m, [0., 0., 0.], [0.2, 2., 0.2], [0.45, 0.2, 0.05]);
            for i in 0..4 {
                let y = i as f32 * 0.4 - 0.8;
                box_part(
                    &mut m,
                    [0., y, 0.],
                    [0.4 + i as f32 * 0.3, 0.2, 0.6],
                    [0.1, 0.5, 0.1],
                );
                box_part(
                    &mut m,
                    [0.4 + i as f32 * 0.15, y, 0.],
                    [0.4, 0.15, 0.3],
                    [1., 0., 0.6],
                );
            }
            for side in [-1., 1.] {
                box_part(
                    &mut m,
                    [side * 0.3, 1., 0.],
                    [0.1, 0.8, 0.1],
                    [0.45, 0.2, 0.05],
                );
            }
        }
        Model::Stool => {
            box_part(&mut m, [0., -0.15, 0.], [0.8, 0.15, 0.8], [1., 0., 0.6]);
            for x in [-0.3, 0.3] {
                for z in [-0.3, 0.3] {
                    box_part(&mut m, [x, -0.6, z], [0.12, 0.8, 0.12], [1., 0., 0.6]);
                }
            }
        }
        Model::ToiletArmy | Model::GrannyArmy | Model::WasherParade | Model::Bodyguards => {}
    }
    m
}

fn formation_template(model: Model) -> Mesh {
    let mut m = Mesh {
        vertices: Vec::with_capacity(320),
    };
    match model {
        Model::Toilet => {
            box_part(&mut m, [0., 0.35, -0.4], [0.8, 0.9, 0.35], [0.9; 3]);
            box_part(&mut m, [0., -0.15, 0.2], [1., 0.3, 1.], [0.95; 3]);
            box_part(&mut m, [0., 0.01, 0.3], [0.6, 0.02, 0.7], [0.05, 0.1, 0.2]);
            for side in [-1., 1.] {
                box_part(&mut m, [side * 0.3, -0.7, 0.], [0.17, 0.6, 0.2], [0.8; 3]);
            }
        }
        Model::Granny => {
            human(&mut m, [0.35, 0.25, 0.5], [0.7, 0.5, 0.4]);
            box_part(&mut m, [0., 1., -0.1], [0.5, 0.3, 0.5], [0.85; 3]);
            box_part(&mut m, [0., -0.25, 0.], [0.9, 0.8, 0.6], [0.4, 0.2, 0.5]);
        }
        Model::Washer => {
            box_part(&mut m, [0., 0., 0.], [1.4, 1.8, 1.], [0.9; 3]);
            box_part(&mut m, [0., -0.1, 0.55], [0.9, 0.9, 0.04], [0.1, 0.3, 0.4]);
        }
        _ => return build_model(model),
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npcs::Role;

    #[test]
    fn every_scene_has_geometry_and_finite_bounded_state_without_allocations() {
        let world = World::new();
        let p = Vec3::new(80., 1., 0.);
        let mut rng = Rng::new(88);
        let mut physics = PhysicsSaboteur::new(&mut rng);
        let mut director = AbsurdDirector::new();
        let mut cars = [Car::new(2, p)];
        let mut npcs = [Npc::new(Role::Resident, 82., 0.)];
        for &model in MODELS {
            let mesh = build_model(model);
            assert!(!mesh.vertices.is_empty());
            assert!(
                mesh.vertices.len() <= 90_000,
                "formation geometry exceeded its budget"
            );
            assert!(mesh
                .vertices
                .iter()
                .all(|v| v.position.iter().all(|p| p.is_finite())));
        }
        let allocations = crate::allocation_probe::count(|| {
            for scene in SCENES {
                assert!(director.force(scene.id, &world, p, Vec3::new(0., 0., 1.)));
                for _ in 0..250 {
                    director.update(
                        &world,
                        &mut cars,
                        &mut npcs,
                        p,
                        p,
                        Vec3::new(0., 0., 1.),
                        &mut physics,
                        &mut rng,
                        0.1,
                    );
                }
            }
        });
        assert_eq!(allocations, 0);
        for p in director.position {
            assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
        }
    }

    #[test]
    fn sausage_swaps_real_npc_and_printer_reuses_exactly_sixty_four_slots() {
        let world = World::new();
        let p = Vec3::new(80., 1., 0.);
        let mut rng = Rng::new(33);
        let mut physics = PhysicsSaboteur::new(&mut rng);
        let mut director = AbsurdDirector::new();
        let mut npcs = [Npc::new(Role::Resident, 100., 0.)];
        director.force(44, &world, p, Vec3::new(0., 0., 1.));
        let sausage = director.position[0];
        let npc = npcs[0].pos;
        director.swap = 0.;
        director.update(
            &world,
            &mut [],
            &mut npcs,
            p,
            p,
            Vec3::new(0., 0., 1.),
            &mut physics,
            &mut rng,
            0.1,
        );
        assert_eq!(director.position[0].x, npc.x);
        assert_eq!(npcs[0].pos.z, sausage.z);
        director.force(94, &world, p, Vec3::new(0., 0., 1.));
        for _ in 0..100 {
            director.update(
                &world,
                &mut [],
                &mut [],
                p,
                p,
                Vec3::new(0., 0., 1.),
                &mut physics,
                &mut rng,
                0.1,
            );
        }
        assert!(director.pigeon_cursor >= PIGEONS.start && director.pigeon_cursor < PIGEONS.end);
        assert!(director.active.iter().filter(|&&v| v).count() <= 65);
        assert!(director.model[PIGEONS.start..PIGEONS.end]
            .iter()
            .all(|&m| m == Model::Pigeon));
    }

    #[test]
    fn borscht_has_eight_seconds_of_antigravity_and_admin_substitutes_stools() {
        let world = World::new();
        let p = Vec3::new(80., 1., 0.);
        let mut rng = Rng::new(33);
        let mut physics = PhysicsSaboteur::new(&mut rng);
        let mut director = AbsurdDirector::new();
        let mut cars = [Car::new(2, p)];
        director.force(5, &world, p, Vec3::new(0., 0., 1.));
        director.update(
            &world,
            &mut cars,
            &mut [],
            p,
            p,
            Vec3::new(0., 0., 1.),
            &mut physics,
            &mut rng,
            0.1,
        );
        assert!(director.gravity_off());
        assert!(physics.car_airborne[0]);
        assert!(director.take_player_lift());
        director.elapsed = 8.;
        assert!(!director.gravity_off());
        director.force(100, &world, p, Vec3::new(0., 0., 1.));
        director.elapsed = 6.;
        assert!(director.stools());
        assert!(director.sky_jelly());
        director.remaining = 0.;
        assert!(!director.stools());
        assert!(!director.sky_jelly());
    }

    #[test]
    fn observed_sign_requests_crouch_and_dumpling_becomes_a_wardrobe() {
        let world = World::new();
        let p = Vec3::new(80., 1., 0.);
        let mut rng = Rng::new(33);
        let mut physics = PhysicsSaboteur::new(&mut rng);
        let mut director = AbsurdDirector::new();
        director.force(10, &world, p, Vec3::new(0., 0., 1.));
        director.update(
            &world,
            &mut [],
            &mut [],
            p,
            p,
            Vec3::new(0., 0., 1.),
            &mut physics,
            &mut rng,
            0.1,
        );
        assert!(director.crouch());
        director.update(
            &world,
            &mut [],
            &mut [],
            p,
            p,
            Vec3::new(0., 0., -1.),
            &mut physics,
            &mut rng,
            0.1,
        );
        assert!(!director.crouch());
        director.force(2, &world, p, Vec3::new(0., 0., 1.));
        director.elapsed = 6.;
        let q = director.position[0];
        let mut npc = [Npc::new(Role::Resident, q.x, q.z)];
        director.update(
            &world,
            &mut [],
            &mut npc,
            p,
            p,
            Vec3::new(0., 0., 1.),
            &mut physics,
            &mut rng,
            0.1,
        );
        assert_eq!(director.model[0], Model::Wardrobe);
    }

    #[test]
    fn source_ids_are_unique_and_invalid_requests_do_not_replace_a_scene() {
        for (i, a) in SCENES.iter().enumerate() {
            assert!(SCENES[..i].iter().all(|b| a.id != b.id));
        }
        let world = World::new();
        let mut director = AbsurdDirector::new();
        assert!(director.force(3, &world, Vec3::default(), Vec3::new(0., 0., 1.)));
        assert!(!director.force(255, &world, Vec3::default(), Vec3::new(0., 0., 1.)));
        assert_eq!(director.source_id(), 3);
    }
}
