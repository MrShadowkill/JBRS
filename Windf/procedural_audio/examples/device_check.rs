use procedural_audio::*;
fn main() -> Result<(), String> {
    let mut audio = Audio::open()?;
    let mut scene = Scene {
        active: true,
        ..Default::default()
    };
    scene.engines[0] = Engine {
        id: 1,
        kind: 6,
        gain: 0.2,
        speed: 40.,
        maximum: 72.,
        throttle: 0.8,
        siren: true,
        ..Default::default()
    };
    for _ in 0..90 {
        audio.pump(&scene)?;
        if audio.queued_bytes() > 3 * 4096 {
            return Err("audio queue exceeded bounded latency".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    audio.pump(&Scene::default())?;
    if audio.queued_bytes() != 0 {
        return Err("paused device retained queued audio".into());
    }
    audio.pump(&scene)?;
    println!(
        "SDL queue/resume check passed: {} queued bytes",
        audio.queued_bytes()
    );
    Ok(())
}
