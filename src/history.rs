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

use flate2::{Compression, GzBuilder};
use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};

use crate::crawl;

pub async fn write(tree: &crawl::Node, dir: &PathBuf) -> Result {
    log::info!("Writing results to tree.json.gz");
    tokio::fs::create_dir_all(&dir).await?;

    let f = fs::File::create(dir.join("tree.json.gz"))?;
    let f = GzBuilder::new()
        .filename("tree.json")
        .write(f, Compression::default());

    let fmter = PrettyFormatter::with_indent(&[]);
    let mut serer = Serializer::with_formatter(f, fmter);

    tree.serialize(&mut serer).map_err(make_err)?;

    Ok(())
}
