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

use actix_web::{web, App, HttpServer, Responder};
use include_flate::flate;

use crate::crawl;

flate!(static RES_INDEX: str from "assets/index.html");

fn res_index(_: web::Path<(String, u32)>) -> impl Responder {
    &*RES_INDEX
}

pub fn run(_tree: crawl::Node, ip: &str, port: u16) -> Result {
    log::info!("Starting web server on {}:{}", ip, port);
    HttpServer::new(|| App::new().service(web::resource("/").route(web::get().to(res_index))))
        .bind(&format!("{}:{}", ip, port))?
        .run()?;
    Ok(())
}
