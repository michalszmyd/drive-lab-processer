use image::{imageops::FilterType::Triangle, ImageError};
use rusty_tesseract::{Args, Image};
use std::{collections::HashMap, ffi::OsStr, path::Path};
use tracing::info;
use url::{ParseError, Url};
const UPLOAD_DIR: &str = "./tmp";

fn default_ocr_args() -> Args {
    Args {
        lang: "pol+eng".to_owned(),
        config_variables: HashMap::from([(
            "tessedit_char_whitelist".into(),
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890 ęóąśłżźćńĘÓĄŚŁŻŹĆŃ?,.!();:[]-%/"
                .into(),
        )]),
        dpi: Some(150),
        psm: Some(6),
        oem: Some(3),
    }
}

#[derive(Debug, derive_more::From)]
pub enum FileToTextError {
    RequestError(reqwest::Error),
    ImageError(ImageError),
    Error(String),
    UrlParseError(ParseError),
}

pub async fn file_to_text(file_url: &str) -> Result<String, FileToTextError> {
    let allowed_types: [&str; 3] = ["png", "jpg", "jpeg"];

    let mut url = Url::parse(file_url)?;
    url.set_query(None);

    let path_file_ext = Path::new(url.as_str())
        .extension()
        .and_then(OsStr::to_str)
        .ok_or(FileToTextError::Error("incorrect extension".to_string()))?
        .to_lowercase();

    if !allowed_types.contains(&path_file_ext.as_str()) {
        return Err(FileToTextError::Error("incorrect file type".to_string()));
    }

    let http_request_image = reqwest::get(file_url).await?;
    let request_image_bytes = http_request_image.bytes().await?;

    let sample_path = format!("{}/{}", UPLOAD_DIR, "last-sample.png");
    let dynamic_image = image::load_from_memory(&request_image_bytes)?
        .adjust_contrast(1000.)
        .resize(1200, 1200, Triangle);

    dynamic_image.save(sample_path).unwrap();

    let img = Image::from_dynamic_image(&dynamic_image).unwrap();

    let output = rusty_tesseract::image_to_string(&img, &default_ocr_args()).unwrap();

    info!("Read: {}", &output);

    Ok(output)
}
