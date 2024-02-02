use std::path::PathBuf;

use crate::app_error::{AppError, AppResult};
use anyhow::{anyhow, Context};
use axum::{
    extract::Path,
    http::{header, HeaderMap},
    routing::get,
    Router,
};
use include_dir::{include_dir, Dir};
use mime_guess::Mime;

pub fn router() -> Router {
    Router::new()
        .route("/assets/railwind.css", get(railwind_generated_css))
        .route("/assets/*path", get(assets))
}

static ASSETS_DIR: Dir = include_dir!("assets");

async fn assets(Path(path): Path<PathBuf>) -> AppResult<(HeaderMap, &'static [u8])> {
    let body = ASSETS_DIR
        .get_file(&path)
        .map(|f| f.contents())
        .ok_or(AppError::NotFound)?;

    let mime_type = get_mime(&path)?;

    let headers = HeaderMap::from_iter(
        [(
            header::CONTENT_TYPE,
            mime_type
                .to_string()
                .parse()
                .context("Failed to convert mime type to header")?,
        )]
        .into_iter(),
    );

    Ok((headers, body))
}

async fn railwind_generated_css() -> AppResult<(HeaderMap, &'static [u8])> {
    let body = include_bytes!(concat!(env!("OUT_DIR"), "/railwind.css"));

    let mime_type = mime_guess::mime::TEXT_CSS;

    let headers = HeaderMap::from_iter(
        [(
            header::CONTENT_TYPE,
            mime_type
                .to_string()
                .parse()
                .context("Failed to convert mime type to header")?,
        )]
        .into_iter(),
    );

    Ok((headers, body))
}

fn get_mime(path: &std::path::Path) -> AppResult<Mime> {
    let ext = path
        .extension()
        .ok_or(anyhow!("Included assets need an extension"))?
        .to_str()
        .ok_or(anyhow!("Path extension had invalid unicode"))?;

    Ok(mime_guess::from_ext(ext)
        .first()
        .context("No mime type guessed")?)
}

#[cfg(test)]
mod tests {
    use include_dir::Dir;

    use super::AppResult;
    use super::{get_mime, ASSETS_DIR};

    #[test]
    fn all_assets_have_a_mime_type() -> AppResult<()> {
        fn check_dir(dir: &Dir) -> AppResult<()> {
            for asset in dir.files() {
                get_mime(&asset.path())?;
            }

            for dir in ASSETS_DIR.dirs() {
                check_dir(dir)?;
            }

            Ok(())
        }

        check_dir(&ASSETS_DIR)?;

        Ok(())
    }
}
