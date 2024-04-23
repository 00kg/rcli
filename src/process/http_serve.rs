use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Serialize;
use tera::{Context, Tera};
use tracing::{info, warn};

use tower_http::services::ServeDir;

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

// #[derive(Serialize,Debug)]
// pub struct TemplateDataItem{
//     name: String,
//     is_dir: bool,
// }

/// 模版数据
// #[derive(Serialize,Debug)]
// pub struct TemplateData {
//     pub list: Vec<TemplateDataItem>,
// }

#[derive(Serialize, Debug)]
pub struct TemplateData {
    pub list: Vec<String>,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on addr {}", path, addr);

    let state = HttpServeState { path: path.clone() };

    let dir_service = ServeDir::new(path)
        .append_index_html_on_directories(true)
        .precompressed_gzip()
        .precompressed_br()
        .precompressed_zstd()
        .precompressed_deflate();

    let router = Router::new()
        .route("/t1/*path", get(file_handler))
        .nest_service("/tower", dir_service)
        .with_state(Arc::new(state));

    // let addr = SocketAddr::from(([0,0,0,0],port));

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, router).await?;

    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, MyError> {
    let p = std::path::Path::new(&state.path).join(path);

    info!("Reading file {:?}", p);

    if !p.exists() {
        return Err(MyError::NotFound("404".to_string()));
    }

    if p.is_dir() {
        let mut entries = tokio::fs::read_dir(p)
            .await
            .map_err(|e| MyError::OtherError(e.to_string()))?;

        let template = Tera::new("templates/*").map_err(|e| MyError::OtherError(e.to_string()))?;

        let mut template_data = TemplateData { list: vec![] };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let filename = entry
                .file_name()
                .into_string()
                .map_err(|_| MyError::OtherError("OsString to String Error".to_string()))?;
            let filename = if entry.path().is_dir() {
                format!("{}/", filename)
            } else {
                filename
            };

            template_data.list.push(filename);
        }

        // println!("template_data: {:?}",template_data);

        let html = template
            .render(
                "indexof.html",
                &Context::from_serialize(&template_data)
                    .map_err(|e| MyError::OtherError(e.to_string()))?,
            )
            .map_err(|e| MyError::OtherError(e.to_string()))?;
        // let html = template.

        // Ok((StatusCode::OK, "SUCC".to_string()))
        Ok((StatusCode::OK, Html(html)).into_response())
    } else {
        match tokio::fs::read_to_string(p).await {
            Ok(content) => Ok((StatusCode::OK, content).into_response()),
            Err(e) => {
                warn!("Error reading file: {:?}", e);
                Err(MyError::OtherError(e.to_string()))
            }
        }
    }
}

#[derive(Debug)]
pub enum MyError {
    NotFound(String),
    OtherError(String),
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        match self {
            MyError::NotFound(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
            MyError::OtherError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_file_handler() -> Result<()> {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });
        let s = file_handler(axum::extract::State(state), Path("Cargo.toml".to_string()))
            .await
            .unwrap()
            .into_response();

        let status = s.status();

        // TODO: Content assert
        assert_eq!(status, StatusCode::OK);
        // assert!(content.contains("anyhow"));

        Ok(())
    }
}
