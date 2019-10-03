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

use std::time::{Duration, Instant};

use futures_timer::Delay;
use futures_util::future::{self, Either};

mod cli;
mod crawl;
mod result;

#[tokio::main]
async fn main() -> Result {
    let args = cli::read()?;

    if !args.dir.is_dir() {
        Err(make_err(
            format!("{}: not a directory", args.dir.display()).as_ref(),
        ))?
    }

    println!("Scanning {}", args.dir.display());
    let epoch = Instant::now();
    let ctx = &crawl::ExploreContext::default();
    let mut ftree = Box::pin(crawl::explore(args.dir, args.shake.get_bytes() as u64, ctx));
    let tree = loop {
        let timeout = Delay::new(Duration::from_millis(100));
        match future::select(timeout, ftree).await {
            Either::Left((res, rtree)) => {
                res?;
                ftree = rtree;
                ctx.display(epoch);
            }
            Either::Right((tree, _)) => {
                eprintln!();
                break tree;
            }
        }
    };

    eprintln!("Writing results to tree.json.gz");
    let f = std::fs::File::create("tree.json.gz")?;
    let f = flate2::GzBuilder::new()
        .filename("tree.json")
        .write(f, flate2::Compression::default());

    let fmter = serde_json::ser::PrettyFormatter::with_indent(&[]);
    let mut serer = serde_json::Serializer::with_formatter(f, fmter);

    use serde::Serialize;
    tree.serialize(&mut serer).map_err(make_err)?;

    Ok(())
}
