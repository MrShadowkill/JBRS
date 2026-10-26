fn main() -> Result<(), String> {
    let w = world::World::new();
    w.audit()?;
    let mut gaps = Vec::new();
    let mut samples = 0;
    for r in &w.roads {
        if r.name == "Mountain tunnel" {
            continue;
        }
        for p in &r.points {
            if terrain::height(p.x, p.z) < -0.2 {
                continue;
            }
            samples += 1;
            let gap = p.y - w.ground_height(p.x, p.z);
            if gap.abs() > 0.2 {
                gaps.push((gap, r.name, p.x, p.z, p.y, w.ground_height(p.x, p.z)));
            }
        }
    }
    gaps.sort_by(|a, b| b.0.abs().total_cmp(&a.0.abs()));
    println!(
        "land centerline samples {samples}; deviations >20cm: {}",
        gaps.len()
    );
    for g in gaps.iter().take(12) {
        println!("{g:?}");
    }
    if !gaps.is_empty() {
        return Err("Land road grading does not match its deck".into());
    }
    Ok(())
}
