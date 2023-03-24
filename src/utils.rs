use crate::Pattern;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::ops::Deref;
use std::{
    path::{Path, PathBuf},
};
use text_colorizer::Colorize;
use uuid::Uuid;


#[derive(Debug, Deserialize, PartialEq, Serialize, Clone, Default)]
pub struct RedactedData {
    redacted_text: String,
    uuid: String,
}

pub(crate) fn get_patterns_from_json(json_file_content: String) -> Result<Vec<Pattern>> {
    serde_json::from_str(&json_file_content).map_err(|err| {
        anyhow!(
            "{}Failed to read patterns from the content of the json file, {err}",
            "ERROR: ".bright_red().bold()
        )
    })
}

pub(crate) fn get_files_from_folder(folder: &str) -> Result<(Vec<PathBuf>, Vec<anyhow::Error>)> {
    let path = Path::new(folder);
    let entries = path.read_dir().map_err(|err| {
        anyhow!(
            "{}Directory: {} cannot be read, err = {err}",
            "ERROR: ".red().bold(),
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
        .filter(|path| {
            match fs::metadata(path) {
                Ok(md) => md.is_file(),
                Err(_) => false,
            }
        })
        .collect();
    let errors = errors
        .into_iter()
        .map(Result::unwrap_err)
        .map(|err| {
            anyhow!(
                "{}Directory Entry cannot be read, err = {err}",
                "ERROR: ".red().bold()
            )
        })
        .collect();
    Ok((entries, errors))
}

pub(crate) fn redact_text_get_data(
    text: &str,
    regex_vec: &[Regex],
) -> Result<(String, Vec<RedactedData>)> {
    let original_text = String::from(text);
    let mut redacted_text = original_text.clone();
    let mut redacted_data: Vec<RedactedData> = Vec::new();
    for regex in regex_vec {
        let matches: Vec<_> = regex.find_iter(&original_text).collect();
        for mat in matches.iter().rev() {
            let uuid = Uuid::new_v4();
            let redacted_str = format!("[REDACTED:{}]", uuid);
            redacted_text.replace_range(mat.start()..mat.end(), &redacted_str);
            redacted_data.push(
                RedactedData {
                    redacted_text: mat.as_str().to_owned(),
                    uuid: uuid.to_string(),
                },
            );
        }
    }
    Ok((redacted_text, redacted_data))
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

    #[test]
    fn test_redact_text_get_data() -> Result<()> {
        use std::collections::HashMap;
        use std::sync::{Arc, Mutex};
        let text = "Hello 123 world!";
        let regex_vec = vec![Regex::new("\\d+").unwrap()];
        let counter = Arc::new(Mutex::new(0));
        let expected_redacted_text =
            "Hello [REDACTED:94f39d29-7a12-4a38-9c50-0ae2eceb5d6a] world!".to_owned();
        let mut expected_redacted_data: HashMap<String, RedactedData> = HashMap::new();
        let uuid = Uuid::parse_str("94f39d29-7a12-4a38-9c50-0ae2eceb5d6a").unwrap();
        expected_redacted_data.insert(
            uuid.to_string(),
            RedactedData {
                redacted_text: "123".to_owned(),
                uuid: uuid.to_string(),
            },
        );
        let (actual_redacted_text, actual_redacted_data) = redact_text_get_data(&text, &regex_vec)?;
        assert_eq!(expected_redacted_text, actual_redacted_text);
        assert_eq!(expected_redacted_data, actual_redacted_data);
        Ok(())
    }
}
