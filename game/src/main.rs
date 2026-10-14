mod doors;
mod escape;
mod game;
mod heists;
mod npcs;
mod quality;
mod world_view;
fn main() {
    if let Err(e) = game::run() {
        eprintln!("JAILBREAK RS: {e}");
        std::process::exit(1);
    }
}
