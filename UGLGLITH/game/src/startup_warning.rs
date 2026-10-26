//! Mandatory per-launch photosensitivity notice. It never reads or writes a save flag.
use renderer_gl21::Renderer;
use std::time::{Duration, Instant};

const WAIT: Duration = Duration::from_secs(10);
const BUTTON: [f32; 4] = [382., 493., 260., 58.];
const LABELS: [&str; 11] = [
    "ОК",
    "ОК (1)",
    "ОК (2)",
    "ОК (3)",
    "ОК (4)",
    "ОК (5)",
    "ОК (6)",
    "ОК (7)",
    "ОК (8)",
    "ОК (9)",
    "ОК (10)",
];
const LINES: [&str; 8] = [
    "Гра містить різке миготіння, спалахи світла,",
    "контрастні кольори та швидкі зміни зображення.",
    "Це може спричинити напад у людей",
    "із фоточутливою епілепсією.",
    "Не грайте, якщо маєте реакцію на миготіння.",
    "При дискомфорті негайно припиніть гру.",
    "Натискаючи ОК, ви підтверджуєте,",
    "що прочитали це попередження.",
];

#[derive(Default)]
struct Warning {
    elapsed: Duration,
}
impl Warning {
    fn advance(&mut self, delta: Duration, foreground: bool) {
        if foreground {
            self.elapsed = self.elapsed.saturating_add(delta).min(WAIT);
        }
    }
    fn ready(&self) -> bool {
        self.elapsed >= WAIT
    }
    fn label(&self) -> &'static str {
        let left = WAIT.saturating_sub(self.elapsed);
        let seconds = left.as_secs() + u64::from(left.subsec_nanos() != 0);
        LABELS[seconds.min(10) as usize]
    }
    fn click(&self, x: f32, y: f32) -> bool {
        self.ready() && gui::contains(BUTTON, x, y)
    }
}

// Three fixed font passes give a thick, steady red face on both GL2.1 and GLES2.
fn bold(r: &mut Renderer, x: f32, y: f32, size: f32, text: &str, color: [f32; 3]) {
    for offset in [0., 0.9, 1.8] {
        r.label(x + offset, y, size, text, color);
    }
}
fn draw(r: &mut Renderer, warning: &Warning) {
    r.begin([0., 0., 0.], 0., 0., 0.);
    r.glitch(0., 0., 0.);
    r.ui();
    r.panel([0., 0., 1024., 576.], [0.015, 0.015, 0.02], 1., 0.);
    let red = [1., 0.08, 0.08];
    bold(r, 72., 32., 2., "UGLGLITH 8.0.7GG", red);
    bold(r, 72., 86., 3., "УВАГА: ФОТОЧУТЛИВА ЕПІЛЕПСІЯ", red);
    for (row, text) in LINES.into_iter().enumerate() {
        bold(r, 72., 155. + row as f32 * 35., 2.05, text, red);
    }
    r.panel(
        BUTTON,
        if warning.ready() {
            [0.45, 0.025, 0.025]
        } else {
            [0.12; 3]
        },
        1.,
        10.,
    );
    bold(
        r,
        if warning.ready() { 478. } else { 448. },
        507.,
        2.6,
        warning.label(),
        if warning.ready() { [1.; 3] } else { [0.55; 3] },
    );
    r.end();
}

/// No smoke/debug bypass: all graphical launches must wait and explicitly accept.
/// Escape, Android Back and window close exit the app instead of opening the game.
pub fn run(r: &mut Renderer) -> Result<bool, String> {
    let mut warning = Warning::default();
    let mut last = Instant::now();
    let mut foreground = true;
    r.capture(false);
    loop {
        let now = Instant::now();
        warning.advance(now.saturating_duration_since(last), foreground);
        last = now;
        while let Some(event) = r.poll() {
            match event.kind() {
                0x100 | 0x101 => return Ok(false),
                0x103 | 0x104 => foreground = false,
                0x105 | 0x106 => foreground = true,
                0x200 => match event.data[12] {
                    2 | 7 => foreground = false,
                    1 | 9 => foreground = true,
                    14 => return Ok(false),
                    _ => {}
                },
                0x300 if !event.repeat() => match event.key() {
                    27 | 1073742094 => return Ok(false),
                    13 | 32 if warning.ready() => return Ok(true),
                    _ => {}
                },
                0x401 if !event.touch_mouse() => {
                    let (x, y) = event.click();
                    if warning.click(
                        x as f32 * 1024. / r.width.max(1) as f32,
                        y as f32 * 576. / r.height.max(1) as f32,
                    ) {
                        return Ok(true);
                    }
                }
                0x700 => {
                    let (_, x, y) = event.finger();
                    if warning.click(x * 1024., y * 576.) {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }
        if foreground {
            draw(r, &warning);
            r.validate()?;
        }
        std::thread::sleep(Duration::from_millis(8));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_ten_seconds_and_explicit_hit_are_required() {
        let mut w = Warning::default();
        assert_eq!(w.label(), "ОК (10)");
        assert!(!w.click(500., 520.));
        w.advance(Duration::from_millis(9_999), true);
        assert!(!w.ready());
        assert_eq!(w.label(), "ОК (1)");
        assert!(!w.click(500., 520.));
        w.advance(Duration::from_millis(1), true);
        assert!(w.ready());
        assert_eq!(w.label(), "ОК");
        assert!(w.click(500., 520.));
        assert!(!w.click(10., 10.));
        assert!(!w.click(f32::NAN, 520.));
    }
    #[test]
    fn background_time_and_previous_launch_do_not_unlock_the_notice() {
        let mut w = Warning::default();
        w.advance(Duration::from_secs(1000), false);
        assert!(!w.ready());
        w.advance(WAIT, true);
        assert!(w.ready());
        let fresh = Warning::default();
        assert!(!fresh.ready());
        assert!(!fresh.click(500., 520.));
    }
    #[test]
    fn countdown_and_hit_testing_do_not_allocate() {
        let mut w = Warning::default();
        let count = crate::allocation_probe::count(|| {
            for _ in 0..700 {
                w.advance(Duration::from_millis(16), true);
                let _ = w.label();
                let _ = w.click(500., 520.);
            }
        });
        assert_eq!(count, 0);
    }
}
