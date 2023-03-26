use crate::Pattern;
use anyhow::{anyhow, Result};
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use crate::RED_ERROR_STRING;

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone, Default)]
pub struct RedactedData {
    unredacted_text: String,
    redacted_text: String,
}

pub(crate) fn get_patterns_from_json(json_file_content: String) -> Result<Vec<Pattern>> {
    serde_json::from_str(&json_file_content).map_err(|err| {
        anyhow!(
            "{}Failed to read patterns from the content of the json file, {err}",
            *RED_ERROR_STRING,
        )
    })
}

pub(crate) fn get_files_from_folder(folder: &str) -> Result<(Vec<PathBuf>, Vec<anyhow::Error>)> {
    let path = Path::new(folder);
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
    let entries = entries
        .into_iter()
        .map(Result::unwrap)
        .map(|entry| entry.path())
        .filter(|path| match fs::metadata(path) {
            Ok(md) => md.is_file(),
            Err(_) => false,
        })
        .collect();
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
    Ok((entries, errors))
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

fn randomize_string(s: &str) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(s.len())
        .map(char::from)
        .collect()
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
