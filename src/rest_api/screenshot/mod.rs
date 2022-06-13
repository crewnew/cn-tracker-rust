use super::{ENDPOINT, HTTP_CLIENT};
use crate::util::random_uuid;
use image::{ImageOutputFormat, RgbImage};
use lazy_static::lazy_static;
use reqwest::blocking::multipart::{Form, Part};
use serde_json::Value;
use std::io::Cursor;

lazy_static! {
    static ref FILES_ENDPOINT: String = format!("{}/files", ENDPOINT.as_str());
}

pub fn send_screenshots(images: &[RgbImage]) -> anyhow::Result<Vec<Value>> {
    #[derive(Deserialize)]
    struct Data {
        data: Value,
    }

    let mut values: Vec<Value> = Vec::with_capacity(images.len());

    for image in images {
        let mut vec: Cursor<Vec<u8>> = Cursor::new(vec![]);
        image.write_to(&mut vec, ImageOutputFormat::Jpeg(85))?;
        let form = Form::new().part(
            "file",
            Part::bytes(vec.into_inner())
                .file_name(format!("{}.jpeg", random_uuid()))
                .mime_str("image/jpeg")?,
        );

        let result = HTTP_CLIENT
            .post(FILES_ENDPOINT.as_str())
            .multipart(form)
            .send()?
            .json::<Data>()?
            .data;

        values.push(result);
    }

    Ok(values)
}
