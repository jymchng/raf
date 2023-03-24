use crate::{utils, RED_ERROR_STRING};
use anyhow::anyhow;
use lopdf::Document;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub(crate) fn redact_txt_and_write_json(
    path: &mut PathBuf,
    regex_vec: &[Regex],
    output_folder: &PathBuf,
) -> anyhow::Result<()> {
    let text = fs::read_to_string(&*path).expect("Failed to read file.");
    let (redacted_text, redacted_data) = utils::redact_text_get_data(&text, &regex_vec)
        .map_err(|err| anyhow!("Unable to get redacted text and the unredacted data, {err}"))?;

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

    file.write_all(redacted_text.as_bytes()).map_err(|err| {
        anyhow!(
            "{}Unable to write the redacted text file, {err}",
            *RED_ERROR_STRING
        )
    })?;

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

    serde_json::to_writer_pretty(unredacted_file, &redacted_data)
        .map_err(|err| anyhow!("{}Failed to write file, {err}", *RED_ERROR_STRING))?;
    anyhow::Ok(())
}

// pub(crate) fn redact_pdf_and_write_json(
//     path: &mut PathBuf,
//     regex_vec: &[Regex],
//     output_folder: &PathBuf,
// ) -> anyhow::Result<()> {
//     let mut pdf = Document::load(path)
//         .map_err(|err| anyhow!("{}Unable to load the pdf, {err}", *RED_ERROR_STRING))?;
//     let page_nums = pdf.get_pages().len();
//     for page_num in 1..page_nums + 1 {
//         let extracted_text = pdf
//             .extract_text([page_num - 1..page_num])
//             .map_err(|err| anyhow!("{}Unable to extract the text, {err}", *RED_ERROR_STRING))?;
//     }
//     let (redacted_text, redacted_data) = utils::redact_text_get_data(&text, &regex_vec)
//         .map_err(|err| anyhow!("Unable to get redacted text and the unredacted data, {err}"))?;

//     let output_path = output_folder.join(path.file_name().ok_or(anyhow!(
//         "{} Unable to join {} with the `file_name` of {}",
//         *RED_ERROR_STRING,
//         output_folder.display(),
//         path.display()
//     ))?);

//     let mut file = fs::File::create(output_path).map_err(|err| {
//         anyhow!(
//             "{}Unable to create the redacted text file, {err}",
//             *RED_ERROR_STRING
//         )
//     })?;

//     file.write_all(redacted_text.as_bytes()).map_err(|err| {
//         anyhow!(
//             "{}Unable to write the redacted text file, {err}",
//             *RED_ERROR_STRING
//         )
//     })?;

//     let mut redacted_json_data_file_path = path
//         .file_stem()
//         .ok_or(anyhow!(
//             "{} Unable to get the `file_stem` of {}\n",
//             *RED_ERROR_STRING,
//             path.display(),
//         ))?
//         .to_os_string();

//     redacted_json_data_file_path.push("-unredact.json");

//     let unredacted_file_path = output_folder.join(redacted_json_data_file_path);

//     let unredacted_file = fs::File::create(unredacted_file_path.clone()).map_err(|err| {
//         anyhow!(
//             "{}Failed to create file {:?}, {err}",
//             *RED_ERROR_STRING,
//             unredacted_file_path
//         )
//     })?;

//     serde_json::to_writer_pretty(unredacted_file, &redacted_data)
//         .map_err(|err| anyhow!("{}Failed to write file, {err}", *RED_ERROR_STRING))?;
//     anyhow::Ok(())
// }
