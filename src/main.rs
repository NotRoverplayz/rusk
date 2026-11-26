// Hello world! This is RUSK, a simple programming language for creating interactive applications and games.




// Uncomment only for final Windows release builds:
// #![windows_subsystem = "windows"]

mod ast;
mod graphics;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod rsml;

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

use rsml::lexer::Lexer as RsmlLexer;
use rsml::parser::Parser as RsmlParser;
use rsml::web::WebRenderer;

const MAGIC: &[u8; 8] = b"RUSK_EXE";

fn window_conf() -> macroquad::prelude::Conf {
    macroquad::prelude::Conf {
        window_title: "RUSK".to_string(),
        window_width: 800,
        window_height: 500,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Build an RSML website
    if args.len() == 3 && args[1] == "build-web" {
        if let Err(err) = build_web(&args[2]) {
            eprintln!("Web build failed: {}", err);
        }
        return;
    }

    // Check for bundled RUSK executable
    let source = if let Some(bundled_source) = read_bundled_script() {
        bundled_source
    } else {
        // Build standalone executable
        if args.len() == 3 && args[1] == "build" {
            if let Err(err) = build_exe(&args[2]) {
                eprintln!("Build failed: {}", err);
            }
            return;
        }

        if args.len() < 2 {
            println!("RUSK 0.2.0");
            println!("Usage:");
            println!("  rusk <file.rusk>");
            println!("  rusk build <file.rusk>");
            println!("  rusk build-web <file.rsml>");
            return;
        }

        match fs::read_to_string(&args[1]) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("Could not read file: {}", error);
                return;
            }
        }
    };

    // Normal RUSK execution pipeline
    let tokens = match Lexer::new(&source).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("Lexer error: {}", error);
            return;
        }
    };

    let program = match Parser::new(tokens).parse() {
        Ok(program) => program,
        Err(error) => {
            eprintln!("Parser error: {}", error);
            return;
        }
    };

    let mut interpreter = Interpreter::new(program);

    if let Err(error) = interpreter.run().await {
        eprintln!("Runtime error: {}", error);
    }
}

fn rsml_script_path(nodes: &[rsml::parser::Node], input_path: &str) -> Option<std::path::PathBuf> {
    fn find(nodes: &[rsml::parser::Node]) -> Option<String> {
        for node in nodes {
            if let rsml::parser::Node::Element(element) = node {
                if let Some(script) = element.attributes.iter()
                    .find(|(name, _)| name == "script")
                    .map(|(_, value)| value.clone())
                {
                    return Some(script);
                }
                if let Some(script) = find(&element.children) {
                    return Some(script);
                }
            }
        }
        None
    }

    let script = find(nodes)?;
    if !script.ends_with(".rusk") {
        return None;
    }

    let base = Path::new(input_path).parent().unwrap_or_else(|| Path::new("."));
    let path = base.join(&script);
    let base = fs::canonicalize(base).ok()?;
    let resolved = if path.exists() {
        fs::canonicalize(&path).ok()?
    } else {
        path
    };
    if resolved.starts_with(&base) {
        Some(resolved)
    } else {
        None
    }
}

fn build_web(input_path: &str) -> std::io::Result<()> {
    println!("RUSK Web Compiler v0.1");
    println!("Compiling: {}", input_path);

    let source = fs::read_to_string(input_path)?;

    // RSML lexer
    let mut lexer = RsmlLexer::new(&source);
    let tokens = lexer.tokenize();

    // RSML parser
    let mut parser = RsmlParser::new(tokens);

    let nodes = parser.parse().map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            err
        )
    })?;

    // Find an associated RUSK script: explicit `script="..."` or sibling main.rusk.
    let mut rusk_source = None;
    let mut script_path = rsml_script_path(&nodes, input_path);
    if script_path.is_none() {
        let main_path = Path::new(input_path).with_file_name("main.rusk");
        if main_path.exists() {
            script_path = Some(main_path);
        }
    }

    if let Some(path) = script_path {
        rusk_source = Some(fs::read_to_string(&path).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Could not read RUSK web script {}: {}", path.display(), error),
            )
        })?);
    }

    if let Some(source) = rusk_source.as_deref() {
        let tokens = Lexer::new(source).tokenize().map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("RUSK web script syntax error: {}", error),
            )
        })?;
        let program = Parser::new(tokens).parse().map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("RUSK web script parse error: {}", error),
            )
        })?;
        rsml::web_scripting::WebRuskCompiler::compile(&program).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("RUSK web script compile error: {}", error),
            )
        })?;
    }

    // Generate HTML
    let html = WebRenderer::render_with_rusk(
        &nodes,
        rusk_source.as_deref(),
    );

    // Save generated website
    let output_path = Path::new(input_path).with_extension("html");

    fs::write(&output_path, html)?;

    println!("Successfully built website!");
    println!("Output: {}", output_path.display());

    Ok(())
}

fn build_exe(script_path: &str) -> std::io::Result<()> {
    let current_exe = env::current_exe()?;
    let script_bytes = fs::read(script_path)?;
    let script_len = script_bytes.len() as u64;

    let out_name = Path::new(script_path)
        .with_extension("exe")
        .file_name()
        .unwrap()
        .to_os_string();

    fs::copy(&current_exe, &out_name)?;

    let mut out_file = OpenOptions::new()
        .append(true)
        .open(&out_name)?;

    out_file.write_all(&script_bytes)?;
    out_file.write_all(&script_len.to_le_bytes())?;
    out_file.write_all(MAGIC)?;

    println!(
        "Successfully built standalone executable: {:?}",
        out_name
    );

    Ok(())
}

fn read_bundled_script() -> Option<String> {
    let current_exe = env::current_exe().ok()?;
    let mut file = File::open(current_exe).ok()?;

    let file_len = file.metadata().ok()?.len();

    if file_len < 16 {
        return None;
    }

    file.seek(SeekFrom::End(-16)).ok()?;

    let mut footer = [0u8; 16];
    file.read_exact(&mut footer).ok()?;

    if &footer[8..16] != MAGIC {
        return None;
    }

    let script_len =
        u64::from_le_bytes(footer[0..8].try_into().ok()?);

    let script_start = file_len.checked_sub(16 + script_len)?;

    file.seek(SeekFrom::Start(script_start)).ok()?;

    let mut script_buf = vec![0u8; script_len as usize];
    file.read_exact(&mut script_buf).ok()?;

    Some(String::from_utf8_lossy(&script_buf).into_owned())
}
