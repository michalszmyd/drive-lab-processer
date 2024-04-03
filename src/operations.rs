use image::imageops::FilterType::Gaussian;
use image::io::Reader as ImageReader;
use rusty_tesseract::{Args, Image};
use std::collections::HashMap;

const UPLOAD_DIR: &str = "./tmp";

fn default_ocr_args() -> Args {
    Args {
        lang: "pol+eng".to_owned(),
        config_variables: HashMap::from([(
            "tessedit_char_whitelist".into(),
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890 ęóąśłżźćńĘÓĄŚŁŻŹĆŃ?,.!"
                .into(),
        )]),
        dpi: Some(150),
        psm: Some(6),
        oem: Some(3),
    }
}

pub fn file_to_text(file_name: &str) -> String {
    let dynamic_image = ImageReader::open(file_name).unwrap().decode().unwrap();
    let dynamic_image = dynamic_image.adjust_contrast(1500.);
    let dynamic_image = dynamic_image.resize(1200, 1200, Gaussian);

    let sample_path = format!("{}/{}", UPLOAD_DIR, "last-sample.png");

    dynamic_image.save(sample_path).unwrap();

    let img = Image::from_dynamic_image(&dynamic_image).unwrap();

    let output = rusty_tesseract::image_to_string(&img, &default_ocr_args()).unwrap();
    println!("The String output for {} is: {:?}", file_name, output);

    output
}
