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

use std::path::PathBuf;

use byte_unit::Byte;

#[derive(Debug)]
pub struct CommandArgs {
    pub dir: PathBuf,
    pub shake: Byte,
}

pub fn read() -> Result<CommandArgs> {
    let app = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::with_name("dir")
                .help("Sets the directory to search")
                .default_value("/")
                .index(1),
        )
        .arg(clap::Arg::with_name("shake")
             .help("Sets the upper bound for leaf shaking. Nodes with size below this value will not be stored.")
             .default_value("1MiB")
             .long("shake"))
        .get_matches();

    let dir = app.value_of("dir").unwrap();
    let dir = PathBuf::from(dir);

    let shake = app.value_of("shake").unwrap();
    let shake = Byte::from_str(shake).map_err(|_| make_err("Failed to parse --shake"))?;

    Ok(CommandArgs { dir, shake })
}
