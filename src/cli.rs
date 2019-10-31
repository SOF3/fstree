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
use std::result::Result as ResultOf;
use std::str::FromStr;

use structopt::StructOpt;

#[derive(Debug)]
pub struct Byte(pub u64);

impl FromStr for Byte {
    type Err = String;
    fn from_str(str: &str) -> ResultOf<Self, Self::Err> {
        let size = byte_unit::Byte::from_str(str)
            .map_err(|err| format!("{:?}", err))?
            .get_bytes() as u64;
        Ok(Self(size))
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "fstree")]
pub struct CommandArgs {
    /// The directory to search
    #[structopt(default_value = "/", parse(from_os_str))]
    pub dir: PathBuf,

    /// The upper bound for leaf shaking. Files below this size will be blackboxed in storage and visualization (but still contribute to total size)
    #[structopt(long, default_value = "1MiB", parse(try_from_str))]
    pub shake: Byte,

    /// Exit directly rather than starting web frontend to view the report
    #[cfg(feature = "web")]
    #[structopt(long)]
    pub no_web: bool,

    /// Run web server only and skip scanning
    #[cfg(feature = "web")]
    #[structopt(long)]
    pub web_only: bool,

    /// The hostmask to start web frontend on
    #[cfg(feature = "web")]
    #[structopt(long, default_value = "127.0.0.1")]
    pub host: String,

    /// The port to start web frontend on
    #[cfg(feature = "web")]
    #[structopt(short, long, default_value = "8000")]
    pub port: u16,

    /// Skips writing history file
    #[cfg(feature = "history")]
    #[structopt(long)]
    pub no_write: bool,

    /// The directory to store history files in
    #[cfg(feature = "history")]
    #[structopt(long, parse(from_os_str))]
    pub history_dir: Option<PathBuf>,

    /// Prevent performing log rotation after writing history file;
    /// this option is ignored if --no-write is passed
    #[cfg(feature = "history")]
    #[structopt(long)]
    pub no_rotate: bool,

    /// Files older than this number of days will be removed during log rotation
    #[cfg(feature = "history")]
    #[structopt(long, default_value = "30")]
    pub rotate_days: u32,
}

pub fn read() -> Result<CommandArgs> {
    let app = CommandArgs::clap()
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"));
    let args = app.get_matches();
    Ok(CommandArgs::from_clap(&args))
}
