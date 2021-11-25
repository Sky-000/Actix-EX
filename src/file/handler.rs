use std::io::Write;

use crate::errors::ServiceError;
use crate::user::model::LoggedUser;
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use futures_util::TryStreamExt as _;
use uuid::Uuid;

pub async fn save_file(user: LoggedUser, mut payload: Multipart) -> Result<HttpResponse, Error> {
    match user.0 {
        None => Err(Error::from(
            HttpResponse::Unauthorized().json(ServiceError::Unauthorized),
        )),
        Some(_) => {
            // iterate over multipart stream
            while let Some(mut field) = payload.try_next().await? {
                // A multipart/form-data stream has to contain `content_disposition`
                let content_disposition = field
                    .content_disposition()
                    .ok_or_else(|| HttpResponse::BadRequest().finish())?;

                let filename = content_disposition.get_filename().map_or_else(
                    || Uuid::new_v4().to_string(),
                    |f| sanitize_filename::sanitize(f),
                );
                let filepath = format!("/root/ex/{}", filename);

                // File::create is blocking operation, use threadpool
                let mut f = web::block(|| std::fs::File::create(filepath)).await?;

                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.try_next().await? {
                    // filesystem operations are blocking, we have to use threadpool
                    f = web::block(move || f.write_all(&chunk).map(|_| f)).await?;
                }
            }

            Ok(HttpResponse::Ok().into())
        }
    }
}

pub fn upload(user: LoggedUser) -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload File To EX</title></head>
        <body>
            <form target="/upload" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    match user.0 {
        None => HttpResponse::Unauthorized().json(ServiceError::Unauthorized),
        Some(_) => HttpResponse::Ok().body(html),
    }
}
