// fstree
// Copyright (C) SOFe
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affer General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#[allow(unused_imports)]
use crate::result::{make_err, Result};

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use actix_web::body::Body;
use actix_web::web::{Data, Json};
use actix_web::{App, HttpResponse, HttpServer, Responder};
use include_flate::flate;
use lazy_static::lazy_static;
use tempdir::TempDir;

use crate::crawl;

pub fn run(current: Option<crawl::Node>, ip: &str, port: u16) -> Result {
    log::debug!("Extracting assets");
    let temp_dir = extract_assets()?;

    log::info!("Starting web server on {}:{}", ip, port);
    serve(ip, port, current, temp_dir)?;

    Ok(())
}

fn extract_assets() -> Result<TempDir> {
    let dir = TempDir::new("fstree")?;
    let tgz = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/web/fstree-web.tar.gz"
    ));
    let cursor = Cursor::new(&tgz[..]);
    let gzd = flate2::read::GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(gzd);
    archive.unpack(&dir)?;
    Ok(dir)
}

type Reports = Arc<RwLock<HashMap<String, crawl::Node>>>;
lazy_static! {
    static ref LATEST_REPORT_NAME: String = "Latest Report".to_string();
}

#[actix_web::get("/")]
fn index() -> impl Responder {
    flate!(static RES: [u8] from "web/index.html");
    HttpResponse::Ok()
        .content_type("text/html")
        .body(Body::Bytes(RES[..].into()))
}

#[actix_web::get("/xhr/has_current")]
fn has_current(reports: Data<Reports>) -> Json<bool> {
    Json(reports.read().unwrap().contains_key(&*LATEST_REPORT_NAME))
}

#[actix_web::get("/xhr/list_reports")]
fn list_reports(_reports: Data<Reports>) -> Json<()> {
    Json(())
}

#[actix_web::post("/xhr/load_report")]
fn load_report(name: String, reports: Data<Reports>) -> Json<bool> {
    let loaded = reports.read().unwrap().contains_key(&*LATEST_REPORT_NAME);
    if loaded {
        return Json(true);
    }

    unimplemented!()
}

fn serve(ip: &str, port: u16, current: Option<crawl::Node>, temp_dir: TempDir) -> Result {
    let mut map = HashMap::new();
    if let Some(current) = current {
        map.insert(LATEST_REPORT_NAME.clone(), current);
    }
    let reports: Reports = Arc::new(RwLock::new(map));
    let temp_dir = Arc::new(temp_dir);

    let server = HttpServer::new(move || {
        App::new()
            .data(Reports::clone(&reports))
            .service(actix_files::Files::new("/pkg", temp_dir.path()))
            .service(index)
            .service(has_current)
            .service(list_reports)
    })
    .bind((ip, port))?;

    thread::spawn(move || {
        thread::sleep(Duration::new(1, 0));
        drop(webbrowser::open(&format!("http://127.0.0.1:{}", port)));
    });

    server.run()?;

    Ok(())
}
