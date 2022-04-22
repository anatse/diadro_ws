mod messages;
mod server;
mod session;
mod wasm_msg;

use std::io::Read;
use std::time::Instant;
use std::{env, fs};

use actix::Addr;
use actix_files::NamedFile;
use actix_web::http::header::ContentEncoding;
use actix_web::web::Path;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};

use actix_web_actors::ws;
use cached::proc_macro::cached;

use actix::prelude::*;
use session::WsChatSession;

/// Cached static files compressed using brotli compression codec. Must be using only for files not larger than 5Mb
#[cached(result = true)]
fn load_file(name: String) -> Result<Vec<u8>> {
    tracing::debug!("start reading file: {}", &name);
    let file = fs::File::open(&name)?;
    let mut buffer = Vec::new();
    let mut input = brotli::CompressorReader::new(file, 8192, 6, 22);
    let size = input.read_to_end(&mut buffer)?;
    tracing::debug!("finish read file: {} size: {}", &name, size);
    Ok(buffer)
}

/// Serves static files
/// ### Argiuments
/// * req - http request
/// * data - configuration data, containing path to static files
async fn index(req: HttpRequest, data: web::Data<String>) -> Result<HttpResponse> {
    let filename = format!("{}/{}", data.as_str(), req.match_info().query("filename"));
    match load_file(filename.clone()) {
        Ok(data) => Ok(HttpResponse::Ok()
            .append_header(ContentEncoding::Brotli)
            .body(data)),
        Err(err) => {
            tracing::error!("{}, file: {}", err, &filename);
            Ok(HttpResponse::NotFound().finish())
        }
    }
}

async fn index_no_compress(req: HttpRequest, data: web::Data<String>) -> Result<NamedFile> {
    let filename = format!("{}/{}", data.as_str(), req.match_info().query("filename"));
    Ok(NamedFile::open(filename)?)
}

/// Entry point for our websocket route
async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    id: Path<String>,
    srv: web::Data<Addr<server::DroServer>>,
) -> Result<HttpResponse> {
    tracing::info!("come to ws route: {:?}", req);
    ws::start(
        WsChatSession {
            id: id.into_inner(),
            hb: Instant::now(),
            name: None,
            addr: srv.get_ref().clone(),
            board: "todo!()".to_owned(),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

    let public_folder = match env::args().nth(1) {
        Some(x) => x,
        None => "diadro/docs".to_string(),
    };

    let data = web::Data::new(public_folder);

    // Create DwoServer
    let dro_srv = server::DroServer::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(web::Data::new(dro_srv.clone()))
            // .wrap(middleware::Compress::default())
            .route("/public/{filename:.*}", web::get().to(index_no_compress))
            .route("/ws/{id}", web::get().to(ws_route))
    })
    .bind(("0.0.0.0", 8081))?
    .workers(num_cpus::get())
    .run()
    .await
}
