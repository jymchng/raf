use crate::utils::RedactedData;
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

pub(crate) fn redact_pdf_and_write_json(
    path: &mut PathBuf,
    regex_vec: &[Regex],
    output_folder: &PathBuf,
) -> anyhow::Result<()> {
    let mut all_redacted_data: Vec<RedactedData> = Vec::new();
    let mut pdf = Document::load(path.clone())
        .map_err(|err| anyhow!("{}Unable to load the pdf, {err}", *RED_ERROR_STRING))?;
    let page_nums: u32 = pdf.get_pages().len().try_into().map_err(|err| {
        anyhow!(
            "{}Unable to convert `usize` into `u32`, {err}",
            *RED_ERROR_STRING
        )
    })?;

    for page_num in 1..page_nums + 1 {
        let extracted_text = pdf
            .extract_text(&[page_num])
            .map_err(|err| anyhow!("{}Unable to extract the text, {err}", *RED_ERROR_STRING))?;
        let (redacted_text, redacted_data) =
            utils::redact_text_get_data(&extracted_text, &regex_vec).map_err(|err| {
                anyhow!(
                    "{}Unable to get redacted text and the unredacted data, {err}",
                    *RED_ERROR_STRING
                )
            })?;
        all_redacted_data.extend(redacted_data);
        pdf.replace_text(page_num, &extracted_text, &redacted_text)
            .map_err(|err| {
                anyhow!(
                    "{}Unable to get replace the text of pdf for page number {page_num}, {err}",
                    *RED_ERROR_STRING
                )
            })?;
    }

    let output_path = output_folder.join(path.file_name().ok_or(anyhow!(
        "{} Unable to join {} with the `file_name` of {}",
        *RED_ERROR_STRING,
        output_folder.display(),
        path.display()
    ))?);

    // let mut file = fs::File::create(output_path).map_err(|err| {
    //     anyhow!(
    //         "{}Unable to create the redacted text file, {err}",
    //         *RED_ERROR_STRING
    //     )
    // })?;

    pdf.save(&output_path).map_err(|err| {
        anyhow!(
            "{}Unable to save pdf to the redacted file path `{}`, {err}",
            *RED_ERROR_STRING,
            output_path.display()
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

    serde_json::to_writer_pretty(unredacted_file, &all_redacted_data)
        .map_err(|err| anyhow!("{}Failed to write file, {err}", *RED_ERROR_STRING))?;
    anyhow::Ok(())
}

pub(crate) fn redact_one_file(path: &mut PathBuf, regex_vec: &[Regex], output_folder: &PathBuf) -> anyhow::Result<()> {
    if let Some(extension) = path.extension() {
        match extension.to_str() {
            Some("txt") => {
                redact_txt_and_write_json(path, &regex_vec, &output_folder)
            }
            Some("pdf") => {
                redact_pdf_and_write_json(path, &regex_vec, &output_folder)
            }
            Some(_) => Err(anyhow!(
                "{}Extension: {:?} not implemented",
                *RED_ERROR_STRING,
                extension
            )),
            None => Err(anyhow!(
                "{}Unable to convert `OsStr` to `str`",
                *RED_ERROR_STRING
            )),
        }
    } else {
        Err(anyhow!(
            "{}Extension of path=`{}` not found",
            *RED_ERROR_STRING,
            path.display()
        ))
    }
}
