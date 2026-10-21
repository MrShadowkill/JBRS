fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .ok_or("Usage: export_editor OUTPUT_DIRECTORY")?;
    jbrs::export_editor_map(std::path::Path::new(&output))
}
