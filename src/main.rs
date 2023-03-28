mod args;
mod redact;
mod utils;

use crate::args::*;
use anyhow::Ok;
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
                    fs::create_dir(&output_folder).expect("Failed to create output folder.");
                };

                let (mut files, dirs, _) = utils::get_files_dirs_from_folder(&path)?;
                dbg!(&dirs);
                if opts.recursive {
                    let dirs = dirs.into_iter().filter(|dir| {
                        // filter out dir name == `redacted`
                        match dir.file_name().unwrap_or_default().to_str() {
                            Some("redacted") => false,
                            _ => true,
                        }
                    });
                    queue.extend(dirs);
                }

                let results: Vec<anyhow::Result<()>> = files
                    .par_iter_mut()
                    .map(|path| redact::redact_one_file(path, &regex_vec, &output_folder))
                    .collect::<Vec<anyhow::Result<()>>>(); // end of for_each

                println!("Processed results: {:?}", results);
            }
            // let output_folder = opts.path.join("redacted");
            // let regex_vec: Vec<Regex> = utils::get_pattern_vec("patterns.json", opts.types)?;

            // if !output_folder.exists() {
            //     fs::create_dir(&output_folder).expect("Failed to create output folder.");
            // };

            // let (mut files, mut dirs, _) = utils::get_files_dirs_from_folder(&opts.path)?;

            // let results: Vec<anyhow::Result<()>> = files
            //     .par_iter_mut()
            //     .map(|path| redact::redact_one_file(path, &regex_vec, &output_folder))
            //     .collect::<Vec<anyhow::Result<()>>>(); // end of for_each

            // println!("Processed results: {:?}", results);
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
