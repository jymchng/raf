mod redact;
mod utils;
mod args;

use anyhow::Ok;
use clap::Parser;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use text_colorizer::{ColoredString, Colorize};
use crate::args::*;

lazy_static! {
    static ref RED_ERROR_STRING: ColoredString = "ERROR: ".red().bold();
}

fn main() -> anyhow::Result<()> {
    let cmd = Opts::parse().cmd;

    match cmd {
        FileOrFolder::Folder(opts) => {
            println!(
                "Folder command executed with path {:?} and types {:?}",
                opts.path, opts.types
            );
            let output_folder = opts.path.join("redacted");
            let regex_vec: Vec<Regex> = utils::get_pattern_vec("patterns.json", opts.types)?;

            if !output_folder.exists() {
                fs::create_dir(&output_folder).expect("Failed to create output folder.");
            };

            let (mut files, _) = utils::get_files_from_folder(&opts.path)?;

            let results: Vec<anyhow::Result<()>> = files
                .par_iter_mut()
                .map(|path| {
                    redact::redact_one_file(path, &regex_vec, &output_folder)
                })
                .collect::<Vec<anyhow::Result<()>>>(); // end of for_each

            println!("Processed results: {:?}", results);
            Ok(())
        }
        FileOrFolder::File(mut opts) => {
            println!(
                "File command executed with path {:?} and types {:?}",
                opts.path, opts.types
            );
            let output_folder = opts.path.parent().unwrap_or(&opts.path).join("redacted");
            let regex_vec: Vec<Regex> = utils::get_pattern_vec("patterns.json", opts.types)?;

            if !output_folder.exists() {
                fs::create_dir(&output_folder).expect("Failed to create output folder.");
            };
            redact::redact_one_file(&mut opts.path, &regex_vec, &output_folder)?;
            Ok(())
        }
    }
}
