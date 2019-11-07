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
use std::env;
use std::time::{Duration, Instant};

use cfg_if::cfg_if;
use futures_util::future::{self, Either};
use tokio::timer;

mod cli;
mod crawl;
mod result;

#[cfg(feature = "history")]
mod history;

#[cfg(feature = "web")]
mod web;

#[tokio::main]
async fn main() -> Result {
    default_env();
    pretty_env_logger::init();

    let args = cli::read()?;

    if !args.dir.is_dir() {
        Err(make_err(
            format!("{}: not a directory", args.dir.display()).as_ref(),
        ))?
    }

    cfg_if! {
        if #[cfg(feature = "web")] {
            if args.web_only {
                if args.no_web {
                    return Err(make_err("--web-only and --no-web are contradictory arguments"));
                }
                web::run(None, &args.host, args.port)?;
            }else {
                let tree = scan(&args).await?;
                if !args.no_web {
                    web::run(Some(tree), &args.host, args.port)?;
                }
            }
        } else {
            scan(&args).await?;
        }
    }

    Ok(())
}

async fn scan(args: &cli::CommandArgs) -> Result<crawl::Node> {
    log::info!("Scanning {}", args.dir.display());
    let epoch = Instant::now();
    let ctx = &crawl::ExploreContext::default();
    let mut ftree = Box::pin(crawl::explore(args.dir.clone(), args.shake.0, ctx));

    #[allow(unused_variables)]
    let tree = loop {
        let timeout = timer::delay_for(Duration::from_millis(100));
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

            if !args.no_rotate {
                if let Err(err) = history::rotate(&history_dir, args.rotate_days).await {
                    log::error!("Error rotating logs: {}", err);
                }
            }
        }
    }

    Ok(tree)
}

fn default_env() {
    if env::var("RUST_LOG") == Err(env::VarError::NotPresent) {
        env::set_var("RUST_LOG", "info");
    }
}
