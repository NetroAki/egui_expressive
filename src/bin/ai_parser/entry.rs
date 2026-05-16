use super::*;

pub(crate) fn run_main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ai-parser <file.ai> [--pretty] [--per-artboard] [--full-canvas]");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let pretty = args.iter().any(|a| a == "--pretty");
    let per_artboard = args.iter().any(|a| a == "--per-artboard");
    let full_canvas = args.iter().any(|a| a == "--full-canvas");

    let result = match parse_ai_file(path) {
        Ok(r) => r,
        Err(e) => {
            let error_result = AiParseResult {
                version: "1.0".to_string(),
                source_file: path.to_string_lossy().to_string(),
                ai_version: String::new(),
                artboards: Vec::new(),
                page_tiles: Vec::new(),
                elements: Vec::new(),
                transform_candidates: Vec::new(),
                errors: vec![e],
            };
            if let Ok(json) = serde_json::to_string(&error_result) {
                println!("{}", json);
            } else {
                println!("{{\"error\": \"not a valid .ai file\"}}");
            }
            return;
        }
    };

    if per_artboard || full_canvas {
        let entries = if full_canvas {
            generate_canvas_output(&result)
        } else {
            generate_per_artboard_output(&result)
        };
        let json = if pretty {
            serde_json::to_string_pretty(&entries)
        } else {
            serde_json::to_string(&entries)
        };
        match json {
            Ok(j) => println!("{}", j),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let json = if pretty {
            serde_json::to_string_pretty(&result)
        } else {
            serde_json::to_string(&result)
        };
        match json {
            Ok(j) => println!("{}", j),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
