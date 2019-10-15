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

use std::fs;
use std::path::PathBuf;

use chrono::offset::Local as LocalTz;
use count_write::CountWrite;
use flate2::{Compression, GzBuilder};
use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};

use crate::crawl;

pub async fn write(tree: &crawl::Node, dir: &PathBuf) -> Result {
    log::info!("Writing history to {}", dir.display());
    tokio::fs::create_dir_all(&dir).await?;

    let date = LocalTz::now().format("%Y-%m-%d_%H-%M-%S");
    let file_name = format!("{}.json", &date);
    let file_path = dir.join(format!("{}.gz", &file_name));
    let f = fs::File::create(&file_path)?;
    let f = GzBuilder::new()
        .filename(file_name.as_str())
        .comment(format!("Filesystem analysis on {}", &date))
        .write(f, Compression::default());
    let cw = CountWrite::from(f);

    let fmter = PrettyFormatter::with_indent(&[]);
    let mut serer = Serializer::with_formatter(cw, fmter);

    tree.serialize(&mut serer).map_err(make_err)?;

    let cw = serer.into_inner();
    let size = cw.count();
    drop(cw);

    log::info!("{} bytes written to {}", size, file_path.display());

    Ok(())
}
