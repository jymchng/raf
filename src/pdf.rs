use crate::utils::{self, redact_text_get_data, RedactedData};
use crate::RED_ERROR_STRING;
use anyhow::{anyhow, Result};
use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use lopdf::{content::Content, Document, Object};
use pdfium_render::{
    page::PdfPage,
    page_object::PdfPageObjectCommon,
    page_objects_common::{PdfPageObjectsCommon, PdfPageObjectsIterator},
    prelude::PdfDocument,
};
use regex::Regex;
use std::borrow::BorrowMut;
use std::collections::BTreeMap;

// pub fn replace_text(pdf_doc: &mut Document, regex_vec: &[Regex]) -> Result<Vec<RedactedData>> {
//     fn collect_text(text: &mut String, encoding: Option<&str>, operands: &[Object]) {
//         for operand in operands.iter() {
//             match *operand {
//                 Object::String(ref bytes, _) => {
//                     let decoded_text = Document::decode_text(encoding, bytes);
//                     text.push_str(&decoded_text);
//                 }
//                 Object::Array(ref arr) => {
//                     collect_text(text, encoding, arr);
//                 }
//                 _ => {}
//             }
//         }
//     }

//     let mut all_redacted_data: Vec<RedactedData> = Vec::new();

//     let pages = pdf_doc.get_pages();
//     for page_number in pages.keys() {
//         let page_id = *pages.get(page_number).ok_or(anyhow!(
//             "{}Page number = {} not found",
//             *RED_ERROR_STRING,
//             page_number
//         ))?;
//         let fonts = pdf_doc.get_page_fonts(page_id);
//         let encodings = fonts
//             .into_iter()
//             .map(|(name, font)| (name, font.get_font_encoding()))
//             .collect::<BTreeMap<Vec<u8>, &str>>();
//         let content_data = pdf_doc.get_page_content(page_id)?;
//         let mut content = Content::decode(&content_data)?;
//         let mut current_encoding = None;
//         for operation in &mut content.operations {
//             match operation.operator.as_ref() {
//                 "Tf" => {
//                     let current_font = operation
//                         .operands
//                         .get(0)
//                         .ok_or_else(|| anyhow!("{}Missing font operand", *RED_ERROR_STRING))?
//                         .as_name()
//                         .map_err(|err| {
//                             anyhow!(
//                                 "{}Unable to get the `current_font`'s name, {err}",
//                                 *RED_ERROR_STRING,
//                             )
//                         })?;
//                     current_encoding = encodings.get(current_font).cloned();
//                 }
//                 "Tj" | "TJ" => {
//                     for bytes in &mut operation.operands.iter().flat_map(Object::as_str_mut) {
//                         let decoded_text = Document::decode_text(current_encoding, &*bytes);
//                         dbg!(&decoded_text);
//                         let (redacted_text, redacted_data) =
//                             redact_text_get_data(&decoded_text, regex_vec).unwrap_or_default();
//                         let encoded_bytes = Document::encode_text(current_encoding, &redacted_text);
//                         all_redacted_data.extend(redacted_data);
//                         *bytes = encoded_bytes;
//                     }
//                 }
//                 _ => {}
//             }
//         }
//         let modified_content = content.encode()?;
//         pdf_doc
//             .change_page_content(page_id, modified_content)
//             .map_err(|err| anyhow!("{}Unable to change content of pdf, {err}", *RED_ERROR_STRING))?;
//     }
//     anyhow::Ok(all_redacted_data)
// }

pub(crate) fn replace_text_pdfium(
    pdf_doc: &mut PdfDocument,
    regex_vec: &[Regex],
) -> Result<Vec<RedactedData>> {
    let mut all_redacted_data: Vec<RedactedData> = Vec::new();

    for (page_index, pdf_page) in pdf_doc.pages_mut().iter().enumerate() {
        for (object_index, mut object) in pdf_page.objects().borrow_mut().iter().enumerate() {

            // For text objects, we take the extra step of outputting the text
            // contained by the object.

            if let Some(object) = object.as_text_object_mut() {
                let object_text = object.text();
                let (redacted_text, redacted_data) =
                        redact_text_get_data(&object_text, regex_vec).unwrap_or_default();
                object.set_text(redacted_text)?;
                all_redacted_data.extend(redacted_data);
            }
        }
    }
    Ok(all_redacted_data)
}

/// First edited
pub fn replace_text(pdf_doc: &mut Document, regex_vec: &[Regex]) -> Result<Vec<RedactedData>> {
    let mut all_redacted_data: Vec<RedactedData> = Vec::new();

    fn collect_text(
        current_encoding: Option<&str>,
        operands: &mut [Object],
        all_redacted_data: &mut Vec<RedactedData>,
        regex_vec: &[Regex],
    ) {
        for operand in operands.iter_mut() {
            match *operand {
                Object::String(ref mut bytes, _) => {
                    let decoded_text = ISO_8859_1.decode(bytes, DecoderTrap::Ignore).unwrap();
                    let (redacted_text, redacted_data) =
                        redact_text_get_data(&decoded_text, regex_vec).unwrap_or_default();
                    let encoded_bytes = ISO_8859_1
                        .encode(&redacted_text, EncoderTrap::Strict)
                        .unwrap();
                    all_redacted_data.extend(redacted_data);
                    *bytes = encoded_bytes;
                    // dbg!(&bytes);
                }
                Object::Array(ref mut arr) => {
                    collect_text(current_encoding, arr, all_redacted_data, regex_vec);
                }
                _ => {}
            }
        }
    }
    let pages = pdf_doc.get_pages();
    for page_number in pages.keys() {
        let page_id = *pages.get(page_number).ok_or_else(|| {
            anyhow!(
                "{}Page number = {} not found",
                *RED_ERROR_STRING,
                page_number
            )
        })?;
        let fonts = pdf_doc.get_page_fonts(page_id);
        let encodings = fonts
            .into_iter()
            .map(|(name, font)| (name, font.get_font_encoding()))
            .collect::<BTreeMap<Vec<u8>, &str>>();
        let content_data = pdf_doc.get_page_content(page_id)?;
        let mut content = Content::decode(&content_data)?;
        let mut current_encoding = None;
        for operation in &mut content.operations {
            match operation.operator.as_ref() {
                "Tf" => {
                    let current_font = operation
                        .operands
                        .get(0)
                        .ok_or_else(|| anyhow!("{}missing font operand", *RED_ERROR_STRING))?
                        .as_name()
                        .map_err(|err| {
                            anyhow!(
                                "{}Unable to get the `current_font`'s name, {err}",
                                *RED_ERROR_STRING,
                            )
                        })?;
                    current_encoding = encodings.get(current_font).cloned();
                }
                "Tj" | "TJ" => {
                    collect_text(
                        current_encoding,
                        &mut operation.operands,
                        &mut all_redacted_data,
                        regex_vec,
                    );
                }
                _ => {}
            }
        }
        let modified_content = content.encode()?;
        pdf_doc
            .change_page_content(page_id, modified_content)
            .map_err(|err| {
                anyhow!(
                    "{}Unable to change content of pdf, {err}",
                    *RED_ERROR_STRING
                )
            })?;
    }
    anyhow::Ok(all_redacted_data)
}
