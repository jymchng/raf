use std::path::PathBuf;
use std::fs;
use regex::Regex;
use std::io::Write;
use crate::{utils, RED_ERROR_STRING};

pub(crate) fn redact_txt_and_write_json(path: &mut PathBuf, regex_vec: &[Regex], output_folder: &PathBuf) {
    let text = fs::read_to_string(&*path).expect("Failed to read file.");
    let (redacted_text, redacted_data) = utils::redact_text_get_data(&text, &regex_vec)
        .expect("Unable to get redacted text and the unredacted data");

    let output_path = output_folder.join(
        path.file_name().expect(
            format!(
                "{} Unable to join {} with the `file_name` of {}",
                *RED_ERROR_STRING,
                output_folder.display(),
                path.display()
            )
            .as_str(),
        ),
    );

    let mut file = fs::File::create(output_path).expect(
        format!(
            "{}Unable to create the redacted text file",
            *RED_ERROR_STRING
        )
        .as_str(),
    );

    file.write_all(redacted_text.as_bytes()).expect(
        format!(
            "{}Unable to write the redacted text file",
            *RED_ERROR_STRING
        )
        .as_str(),
    );

    let mut redacted_json_data_file_path = path
        .file_stem()
        .expect(
            format!(
                "{} Unable to get the `file_stem` of {}\n",
                *RED_ERROR_STRING,
                path.display(),
            )
            .as_str(),
        )
        .to_os_string();
    redacted_json_data_file_path.push("-unredact.json");
    let unredacted_file_path = output_folder.join(redacted_json_data_file_path);
    let unredacted_file = fs::File::create(unredacted_file_path.clone()).expect(
        format!(
            "{}Failed to create file {:?}",
            *RED_ERROR_STRING, unredacted_file_path
        )
        .as_str(),
    );
    serde_json::to_writer_pretty(unredacted_file, &redacted_data)
        .expect(format!("{}Failed to write file", *RED_ERROR_STRING,).as_str());
}
