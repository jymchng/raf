use crate::utils;
use crate::RED_ERROR_STRING;
use anyhow::{anyhow, Result};
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone, Default)]
pub struct RedactedData {
    unredacted_text: String,
    redacted_text: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone, Default)]
pub struct Pattern {
    pattern: String,
    #[serde(rename = "type")]
    types: Vec<String>,
}

pub(crate) fn get_patterns_from_json(json_file_content: String) -> Result<Vec<Pattern>> {
    serde_json::from_str(&json_file_content).map_err(|err| {
        anyhow!(
            "{}Failed to read patterns from the content of the json file, {err}",
            *RED_ERROR_STRING,
        )
    })
}

pub(crate) fn get_files_dirs_from_folder(
    path: &PathBuf,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<anyhow::Error>)> {
    let entries = path.read_dir().map_err(|err| {
        anyhow!(
            "{}Directory: {} cannot be read, err = {err}",
            *RED_ERROR_STRING,
            path.display()
        )
    })?;
    let (entries, errors): (Vec<_>, Vec<_>) = entries
        // https://doc.rust-lang.org/rust-by-example/error/iter_result.html
        .into_iter()
        .partition(Result::is_ok);
    let (files, dirs): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .map(Result::unwrap)
        .map(|entry| entry.path())
        .partition(|p| Path::is_file(p));

    let errors = errors
        .into_iter()
        .map(Result::unwrap_err)
        .map(|err| {
            anyhow!(
                "{}Directory Entry cannot be read, err = {err}",
                *RED_ERROR_STRING
            )
        })
        .collect();
    Ok((files, dirs, errors))
}

pub(crate) fn redact_text_get_data(
    text: &str,
    regex_vec: &[Regex],
) -> Result<(String, Vec<RedactedData>)> {
    let mut redacted_text = String::from(text);
    let mut redacted_data: Vec<RedactedData> = Vec::new();
    for regex in regex_vec {
        let matches: Vec<_> = regex.find_iter(text).collect();
        for mat in matches.iter().rev() {
            let randomized_str = randomize_string(mat.as_str());
            let redacted_str = "[REDACTED:".to_string() + &randomized_str + "]";
            redacted_text.replace_range(mat.start()..mat.end(), &redacted_str);
            redacted_data.push(RedactedData {
                unredacted_text: mat.as_str().to_owned(),
                redacted_text: randomized_str,
            });
        }
    }
    Ok((redacted_text, redacted_data))
}

pub(crate) fn randomize_string(s: &str) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(s.len())
        .map(char::from)
        .collect()
}

pub(crate) fn get_pattern_vec(pattern_file: &str, types: Vec<String>) -> Result<Vec<Regex>> {
    let patterns_json_content = fs::read_to_string(pattern_file)
        .map_err(|err| anyhow!("{}Cannot open {pattern_file}, {err}", *RED_ERROR_STRING))?;

    let patterns = utils::get_patterns_from_json(patterns_json_content)?;

    let filtered_patterns: Vec<Pattern> = patterns
        .into_iter()
        .filter(|p| p.types.iter().any(|t| types.contains(&t)))
        .collect();

    let regex_vec: Vec<Regex> = filtered_patterns
        .iter()
        .map(|p| Regex::new(&p.pattern).expect("Invalid regex pattern."))
        .collect();
    Ok(regex_vec)
}

pub(crate) fn write_redacted_data_json(
    all_redacted_data: Vec<RedactedData>,
    path: &PathBuf,
    output_folder: &PathBuf,
) -> anyhow::Result<()> {
    let mut redacted_json_data_file_path = path
        .file_stem()
        .ok_or(anyhow!(
            "{} Unable to get the `file_stem` of {}\n",
            *RED_ERROR_STRING,
            path.display(),
        ))?
        .to_os_string();

    redacted_json_data_file_path.push("-unredact.json");

    let unredacted_file_path = output_folder.join(redacted_json_data_file_path);

    let unredacted_file = fs::File::create(unredacted_file_path.clone()).map_err(|err| {
        anyhow!(
            "{}Failed to create file {:?}, {err}",
            *RED_ERROR_STRING,
            unredacted_file_path
        )
    })?;

    serde_json::to_writer_pretty(unredacted_file, &all_redacted_data)
        .map_err(|err| anyhow!("{}Failed to write file, {err}", *RED_ERROR_STRING))?;
    anyhow::Ok(())
}

pub(crate) fn write_redacted_text<S: AsRef<[u8]>>(
    redacted_text: S,
    path: &PathBuf,
    output_folder: &PathBuf,
) -> anyhow::Result<()> {
    let output_path = output_folder.join(path.file_name().ok_or(anyhow!(
        "{} Unable to join {} with the `file_name` of {}",
        *RED_ERROR_STRING,
        output_folder.display(),
        path.display()
    ))?);

    let mut file = fs::File::create(output_path).map_err(|err| {
        anyhow!(
            "{}Unable to create the redacted text file, {err}",
            *RED_ERROR_STRING
        )
    })?;

    file.write_all(redacted_text.as_ref()).map_err(|err| {
        anyhow!(
            "{}Unable to write the redacted text file, {err}",
            *RED_ERROR_STRING
        )
    })?;
    anyhow::Ok(())
}

#[derive(Debug)]
struct AnyhowErrVec(Vec<anyhow::Error>);

impl PartialEq for AnyhowErrVec {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(other.0.iter())
                .all(|(a, b)| a.to_string() == b.to_string())
    }
}

impl From<Vec<anyhow::Error>> for AnyhowErrVec {
    fn from(vec: Vec<anyhow::Error>) -> Self {
        AnyhowErrVec(vec)
    }
}

impl Deref for AnyhowErrVec {
    type Target = Vec<anyhow::Error>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Pattern;
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn test_get_patterns_from_json() -> Result<()> {
        let json_file = r#"
        [
            {
                "pattern": "\\d+",
                "type": ["pattern1", "pattern12"]
            },
            {
                "pattern": "\\w+",
                "type": ["email", "emails"]
            }
        ]
        "#;
        let expected_patterns = vec![
            Pattern {
                types: vec!["pattern1".to_string(), "pattern12".to_string()],
                pattern: "\\d+".to_owned(),
            },
            Pattern {
                types: vec!["email".to_owned(), "emails".to_string()],
                pattern: "\\w+".to_owned(),
            },
        ];
        let actual_patterns = get_patterns_from_json(json_file.to_owned())?;
        assert_eq!(expected_patterns, actual_patterns);
        Ok(())
    }

    #[test]
    fn test_get_files_from_folder() -> Result<()> {
        let folder = "./tests/test_files";
        let (expected_entries, expected_errors): (Vec<PathBuf>, Vec<anyhow::Error>) = (
            vec![
                PathBuf::from("./tests/test_files/file1.txt"),
                PathBuf::from("./tests/test_files/file2.txt"),
            ],
            vec![],
        );
        let (actual_entries, actual_errors) = get_files_from_folder(folder)?;
        assert_eq!(expected_entries, actual_entries);
        assert_eq!(
            AnyhowErrVec::from(expected_errors),
            AnyhowErrVec::from(actual_errors)
        );
        Ok(())
    }

    #[test]
    fn test_get_output_file_path() -> Result<()> {
        let file_path = PathBuf::from("path/to/file.txt");
        let expected_output_path = PathBuf::from("file.txt-unredact.json");
        let actual_output_path = get_output_file_path(&file_path)?;
        assert_eq!(expected_output_path, actual_output_path);
        Ok(())
    }
}


