mod args;
mod pdf;
mod redact;
mod utils;

use crate::args::*;
use anyhow::{Ok, anyhow};
use clap::Parser;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use text_colorizer::{ColoredString, Colorize};

lazy_static! {
    static ref RED_ERROR_STRING: ColoredString = "ERROR: ".red().bold();
}

fn main() -> anyhow::Result<()> {
    let cmd = Opts::parse().cmd;
    let mut queue = VecDeque::<PathBuf>::new();

    match cmd {
        FileOrFolder::Folder(opts) => {
            println!(
                "Folder command executed with path {:?}, recursive {:?} and types {:?}",
                opts.path, opts.recursive, opts.types
            );

            let regex_vec: Vec<Regex> = utils::get_pattern_vec("patterns.json", opts.types)?;
            queue.push_back(opts.path);

            while let Some(path) = queue.pop_front() {
                let output_folder = path.join("redacted");

                if !output_folder.exists() {
                    fs::create_dir(&output_folder).map_err(|err| {
                        anyhow!(
                            "{}Failed to create output folder, {err}.",
                            *RED_ERROR_STRING
                        )
                    })?;
                };

                let (files, dirs, _) = utils::get_files_dirs_from_folder(&path)?;
                if opts.recursive {
                    let dirs = dirs.into_iter().filter(|dir| {
                        // filter out dir name == `redacted`
                        matches!(dir.file_name().unwrap_or_default().to_str(), Some("redacted"))
                    });
                    queue.extend(dirs);
                }

                let results: Vec<anyhow::Result<()>> = files
                    .par_iter()
                    .map(|path| redact::redact_one_file(path, &regex_vec, &output_folder))
                    .collect::<Vec<anyhow::Result<()>>>(); // end of for_each

                println!("Processed results: {:?}", results);
                println!("A folder named `redacted` at {} has been created containing all the redacted files in {}",
                        output_folder.display(),
                        path.display());
            }
            Ok(())
        }
        FileOrFolder::File(opts) => {
            println!(
                "File command executed with path {:?} and types {:?}",
                opts.path, opts.types
            );
            let output_folder = opts.path.parent().unwrap_or(&opts.path).join("redacted");
            let regex_vec: Vec<Regex> = utils::get_pattern_vec("patterns.json", opts.types)?;

            if !output_folder.exists() {
                fs::create_dir(&output_folder).expect("Failed to create output folder.");
            };
            redact::redact_one_file(&opts.path, &regex_vec, &output_folder)?;
            Ok(())
        }
    }
}
