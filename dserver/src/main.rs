mod messages;
mod server;
mod session;
mod wasm_msg;

use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Instant;
use std::{env, fs};

use actix::Addr;
use actix_web::http::header::ContentEncoding;
use actix_web::web::Path;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};

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

// async fn index_no_compress(req: HttpRequest, data: web::Data<String>) -> Result<NamedFile> {
//     let filename = format!("{}/{}", data.as_str(), req.match_info().query("filename"));
//     Ok(NamedFile::open(filename)?)
// }

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
    tracing_subscriber::fmt().init();
    // let _ = tracing::subscriber::set_global_default(sbr)
    //     .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

    let public_folder = match env::args().nth(1) {
        Some(x) => x,
        None => "diadro/docs".to_string(),
    };

    let data = web::Data::new(public_folder);

    // Get pem file with private key
    let tls_config = load_rustls_config();

    // Create DwoServer
    let dro_srv = server::DroServer::new().start();
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(web::Data::new(dro_srv.clone()))
            .wrap(middleware::Compress::default())
            .route("/public/{filename:.*}", web::get().to(index))
            .route("/ws/{id}", web::get().to(ws_route))
    })
    .bind_rustls(("0.0.0.0", 8083), tls_config)?
    .workers(num_cpus::get_physical())
    .run()
    .await
}

fn load_rustls_config() -> rustls::ServerConfig {
    let pk_file = env::var("PK_FILE").unwrap_or_else(|err| {
        tracing::warn!(
            "Error reading PK_FILE. Standatd value will be used. Error: {}",
            err
        );
        "./keys/key.pem".to_string()
    });

    let cert_file = env::var("PK_FILE").unwrap_or_else(|err| {
        tracing::warn!(
            "Error reading CERT_FILE. Standatd value will be used. Error: {}",
            err
        );
        "./keys/cert.pem".to_string()
    });

    // init server config builder with safe defaults
    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open(cert_file).unwrap());
    let key_file = &mut BufReader::new(File::open(pk_file).unwrap());

    // convert files to key/cert objects
    let cert_chain = rustls_pemfile::certs(cert_file)
        .unwrap()
        .into_iter()
        .map(rustls::Certificate)
        .collect();
    let mut keys: Vec<rustls::PrivateKey> = rustls_pemfile::pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(rustls::PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}
