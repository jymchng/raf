mod utils;
mod redact;

use anyhow::{anyhow, Ok};
use clap::{App, Arg};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use text_colorizer::{ColoredString, Colorize};
use uuid::Uuid;
use std::dbg;

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone, Default)]
struct Pattern {
    pattern: String,
    #[serde(rename = "type")]
    types: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone, Default)]
struct Redacted {
    uuid: String,
    text: String,
}

lazy_static! {
    static ref RED_ERROR_STRING: ColoredString = "ERROR: ".red().bold();
}

fn main() -> anyhow::Result<()> {
    let matches = App::new("text-redactor")
        .version("1.0")
        .author("Your Name <you@example.com>")
        .about("Redacts text matching given regex patterns.")
        .arg(
            Arg::with_name("input")
                .short('i')
                .long("input")
                .value_name("INPUT")
                .help("Sets the input folder to use")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("types")
                .short('t')
                .long("types")
                .value_name("TYPES")
                .help("Types of patterns to use for redaction")
                .takes_value(true)
                .multiple(true)
                .required(true),
        )
        .try_get_matches()
        .unwrap_or_else(|e| e.exit());

    let input_folder = matches.value_of("input").ok_or(anyhow!(
        "{}`input` CLI Parameter not found",
        "Error".bright_red().bold()
    ))?;
    let pattern_file = "patterns.json";
    let types: Vec<String> = matches
        .get_many::<String>("types")
        .ok_or(anyhow!(
            "{}`types` CLI Parameter not found",
            "Error".bright_red().bold()
        ))?
        .map(|s| s.to_owned())
        .collect();

    // Load regex patterns from JSON file
    let patterns_json_content = fs::read_to_string(pattern_file)
        .map_err(|err| anyhow!("{}Cannot open {pattern_file}", *RED_ERROR_STRING))?;

    let patterns: Vec<Pattern> = utils::get_patterns_from_json(patterns_json_content)?;

    // Filter patterns based on types
    let filtered_patterns: Vec<Pattern> = patterns
        .into_iter()
        .filter(|p| p.types.iter().any(|t| types.contains(&t)))
        .collect();

    // Compile regex patterns
    let regex_vec: Vec<Regex> = filtered_patterns
        .iter()
        .map(|p| Regex::new(&p.pattern).expect("Invalid regex pattern."))
        .collect();

    // Create output folder
    let output_folder = Path::new(input_folder).join("redacted");
    if !output_folder.exists() {
        fs::create_dir(&output_folder).expect("Failed to create output folder.");
    };

    let (mut files, errors) = utils::get_files_from_folder(input_folder)?;

    let results = Arc::new(Mutex::new(Vec::<anyhow::Result<()>>::new()));

    files.par_iter_mut().for_each(|path| {
        
        if let Some(extension) = path.extension() {
            if extension == "txt" {
                redact::redact_txt_and_write_json(path, &regex_vec, &output_folder);
            } else {
                println!(
                    "{}INVALID EXTENSION: {} - Not yet implemented",
                    *RED_ERROR_STRING,
                    extension.to_string_lossy(),
                );
                std::process::exit(1);
            };
            ()
        } else {
            println!("{}EXTENSION not found", *RED_ERROR_STRING);
        }
    }); // end of for_each
    Ok(())
}
