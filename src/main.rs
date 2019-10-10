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

use std::borrow::Cow;
use std::time::{Duration, Instant};

use futures_timer::Delay;
use futures_util::future::{self, Either};

mod cli;
mod crawl;
mod result;

#[cfg(feature = "history")]
mod history;

#[cfg(feature = "web")]
mod web;

#[tokio::main]
async fn main() -> Result {
    pretty_env_logger::init();

    let args = cli::read()?;

    if !args.dir.is_dir() {
        Err(make_err(
            format!("{}: not a directory", args.dir.display()).as_ref(),
        ))?
    }

    log::info!("Scanning {}", args.dir.display());
    let epoch = Instant::now();
    let ctx = &crawl::ExploreContext::default();
    let mut ftree = Box::pin(crawl::explore(args.dir, args.shake.0, ctx));

    #[allow(unused_variables)]
    let tree = loop {
        let timeout = Delay::new(Duration::from_millis(100));
        match future::select(timeout, ftree).await {
            Either::Left(((), rtree)) => {
                ftree = rtree;
                ctx.display(epoch);
            }
            Either::Right((tree, _)) => {
                eprintln!();
                break tree;
            }
        }
    };

    #[cfg(feature = "history")]
    {
        if !args.no_write {
            let history_dir = if let Some(dir) = &args.history_dir {
                Cow::Borrowed(dir)
            } else {
                Cow::Owned(
                    dirs::home_dir()
                        .expect("Failed to get home directory")
                        .join(".fstree/history"),
                )
            };
            history::write(&tree, &history_dir).await?;
            // history::rotate()?;
        }
    }

    #[cfg(feature = "web")]
    {
        if !args.no_web {
            web::run(tree, &args.host, args.port)?;
        }
    }

    Ok(())
}
