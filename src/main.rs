mod utils;
mod redact;

use anyhow::{anyhow, Ok};
use clap::{App, Arg};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use text_colorizer::{ColoredString, Colorize};

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

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone, Default)]
enum FileOrFolder {
    File,
    Folder,
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
            Arg::with_name("folder")
                .short('d')
                .long("folder")
                .value_name("FOLDER")
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

    let input_folder = matches.value_of("folder").ok_or(anyhow!(
        "{}`folder` CLI Parameter not found",
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
        .map_err(|err| anyhow!("{}Cannot open {pattern_file}, {err}", *RED_ERROR_STRING))?;

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

    let (mut files, _) = utils::get_files_from_folder(input_folder)?;

    let results: Vec<anyhow::Result<()>> = files.par_iter_mut().map(|path| {
        
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("txt") => redact::redact_txt_and_write_json(path, &regex_vec, &output_folder),
                Some(_) => Err(anyhow!("{}Extension: {:?} not implemented", *RED_ERROR_STRING, extension)),
                None => Err(anyhow!("{}Unable to convert `OsStr` to `str`", *RED_ERROR_STRING)),
            }
        } else {
            Err(anyhow!("{}Extension of path=`{}` not found", *RED_ERROR_STRING, path.display()))
        }
    }).collect::<Vec<anyhow::Result<()>>>(); // end of for_each
    println!(
        "Processing results: {:?}", results
    );
    Ok(())
}
