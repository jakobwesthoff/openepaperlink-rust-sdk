use crate::client::Client;
use crate::{Error, Mac, UploadImageOptions};

impl Client {
    /// Upload a JPEG image to be displayed on a tag.
    pub async fn upload_image(
        &self,
        mac: &Mac,
        image_bytes: Vec<u8>,
        options: &UploadImageOptions,
    ) -> Result<(), Error> {
        let url = self.url("/imgupload");

        let image_part = reqwest::multipart::Part::bytes(image_bytes)
            .file_name("image.jpg")
            .mime_str("image/jpeg")
            .expect("image/jpeg is a valid MIME type");

        let mut form = reqwest::multipart::Form::new()
            .text("mac", mac.to_string())
            .part("file", image_part);

        if let Some(dither) = options.dither {
            form = form.text("dither", dither.to_string());
        }
        if let Some(ref alias) = options.alias {
            form = form.text("alias", alias.clone());
        }
        if let Some(rotate) = options.rotate {
            form = form.text("rotate", rotate.to_string());
        }
        if let Some(lut) = options.lut {
            form = form.text("lut", lut.to_string());
        }
        if let Some(invert) = options.invert {
            form = form.text("invert", if invert { "1" } else { "0" }.to_string());
        }
        if let Some(mode) = options.content_mode {
            form = form.text("contentmode", mode.to_u8().to_string());
        }
        if let Some(ttl) = options.ttl {
            form = form.text("ttl", ttl.to_string());
        }

        let body = self
            .http
            .post(&url)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        self.check_response_body(&body)
    }

    /// Upload a JSON template to be rendered on a tag.
    pub async fn upload_json_template(
        &self,
        mac: &Mac,
        json: &str,
        ttl: Option<u32>,
    ) -> Result<(), Error> {
        let url = self.url("/jsonupload");
        let mut params = vec![
            ("mac".to_string(), mac.to_string()),
            ("json".to_string(), json.to_string()),
        ];
        if let Some(ttl) = ttl {
            params.push(("ttl".to_string(), ttl.to_string()));
        }

        let body = self
            .http
            .post(&url)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        self.check_response_body(&body)
    }
}
