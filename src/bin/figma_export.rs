//! CLI binary for exporting Figma design tokens to Rust code.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("Usage: figma-export <tokens.json>");

    let json = std::fs::read_to_string(path).expect("Failed to read file");

    match egui_expressive::figma::figma_tokens_to_rust(&json) {
        Ok(code) => print!("{}", code),
        Err(e) => eprintln!("Error: {}", e),
    }
}
