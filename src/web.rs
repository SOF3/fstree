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

use std::io::Cursor;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use actix_web::{body::Body, web, App, HttpResponse, HttpServer};
use include_flate::flate;
use tempdir::TempDir;

use crate::crawl;

pub fn run(current: Option<&crawl::Node>, ip: &str, port: u16) -> Result {
    log::debug!("Extracting assets");
    let temp_dir = extract_assets()?;

    log::info!("Starting web server on {}:{}", ip, port);
    serve(ip, port, temp_dir)?;

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

fn serve(ip: &str, port: u16, temp_dir: TempDir) -> Result {
    let temp_dir = Arc::new(temp_dir);

    let server = HttpServer::new(move || {
        App::new()
            .service(actix_files::Files::new("/pkg", temp_dir.path()))
            .route(
                "/",
                web::get().to(|| {
                    flate!(static RES: [u8] from "web/index.html");
                    HttpResponse::Ok()
                        .content_type("text/html")
                        .body(Body::Bytes(RES[..].into()))
                }),
            )
    })
    .bind(&format!("{}:{}", ip, port))?;

    thread::spawn(move || {
        thread::sleep(Duration::new(1, 0));
        webbrowser::open(&format!("http://127.0.0.1:{}", port)).ok();
    });

    server.run()?;

    Ok(())
}
