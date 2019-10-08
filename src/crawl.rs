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
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::Metadata;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use derive_more::AddAssign;
use futures_util::future::{join_all, BoxFuture, FutureExt};
use futures_util::stream::StreamExt;
use maplit::hashmap;
#[cfg(feature = "history")]
use serde::{Deserialize, Serialize};
use static_assertions::assert_impl_all;
use terminal_size::terminal_size;
use tokio::io;
use tokio_fs as fs;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "history", derive(Serialize, Deserialize))]
pub enum Node {
    File {
        name: StringRef,
        extension: StringRef,
        size: Size,
    },
    Dir {
        name: StringRef,
        children: Vec<Node>,
        stats: NodeStats,
    },
    Link {
        name: StringRef,
        size: Size,
    },
    Error {
        name: Option<StringRef>,
        error: StaticError,
    },
    BlockDevice {
        name: StringRef,
        size: Size,
    },
    CharDevice {
        name: StringRef,
        size: Size,
    },
    NamedPipe {
        name: StringRef,
        size: Size,
    },
    UnixSocket {
        name: StringRef,
        size: Size,
    },
    Other {
        name: StringRef,
        extension: StringRef,
        size: Size,
    },
}

impl From<io::Error> for Node {
    fn from(error: io::Error) -> Node {
        Node::Error {
            name: None,
            error: error.into(),
        }
    }
}

impl Node {
    pub fn stats(&self) -> NodeStats {
        match self {
            Node::File {
                size, extension, ..
            } => {
                let typed = TypedStats {
                    files: AggStats {
                        count: 1,
                        size: *size,
                    },
                    ..Default::default()
                };
                NodeStats {
                    total: typed,
                    by_extension: vec![(FileTypeExt::File(Arc::clone(extension)), typed)],
                }
            }
            Node::Dir { stats, .. } => stats.clone(),
            Node::Link { size, .. } => make_other_stats(*size, FileTypeExt::Link),
            Node::BlockDevice { size, .. } => make_other_stats(*size, FileTypeExt::BlockDevice),
            Node::CharDevice { size, .. } => make_other_stats(*size, FileTypeExt::CharDevice),
            Node::NamedPipe { size, .. } => make_other_stats(*size, FileTypeExt::NamedPipe),
            Node::UnixSocket { size, .. } => make_other_stats(*size, FileTypeExt::UnixSocket),
            Node::Error { .. } => {
                let typed = TypedStats {
                    errors: 1,
                    ..Default::default()
                };
                NodeStats {
                    total: typed,
                    by_extension: vec![(FileTypeExt::Error, typed)],
                }
            }
            Node::Other {
                size, extension, ..
            } => {
                let typed = TypedStats {
                    others: AggStats {
                        count: 1,
                        size: *size,
                    },
                    ..Default::default()
                };
                NodeStats {
                    total: typed,
                    by_extension: vec![(FileTypeExt::Other(Arc::clone(extension)), typed)],
                }
            }
        }
    }
}

fn make_other_stats(size: Size, fte: FileTypeExt) -> NodeStats {
    let typed = TypedStats {
        others: AggStats { count: 1, size },
        ..Default::default()
    };
    NodeStats {
        total: typed,
        by_extension: vec![(fte, typed)],
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "history", derive(Serialize, Deserialize))]
pub struct StaticError(String);

impl From<io::Error> for StaticError {
    fn from(err: io::Error) -> Self {
        Self(err.to_string())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "history", derive(Serialize, Deserialize))]
pub enum FileTypeExt {
    File(StringRef),
    Dir,
    Link,
    BlockDevice,
    CharDevice,
    NamedPipe,
    UnixSocket,
    Error,
    Other(StringRef),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "history", derive(Serialize, Deserialize))]
pub struct NodeStats {
    pub total: TypedStats,
    pub by_extension: Vec<(FileTypeExt, TypedStats)>,
}

#[derive(Debug, Clone, Copy, Default, AddAssign)]
#[cfg_attr(feature = "history", derive(Serialize, Deserialize))]
pub struct TypedStats {
    pub files: AggStats,
    pub dirs: AggStats,
    pub others: AggStats,
    pub errors: usize,
}

#[derive(Debug, Clone, Copy, Default, AddAssign)]
#[cfg_attr(feature = "history", derive(Serialize, Deserialize))]
pub struct AggStats {
    pub count: usize,
    pub size: Size,
}

#[derive(Debug, Clone, Copy, Default, AddAssign)]
#[cfg_attr(feature = "history", derive(Serialize, Deserialize))]
pub struct Size {
    pub real: u64,
    pub content: u64,
}

pub type StringRef = Arc<str>;
type StringPool = RwLock<HashSet<StringRef>>;

#[derive(Debug, Default)]
pub struct ExploreContext {
    pub pool: StringPool,
    pub pending: AtomicUsize,
    pub complete: AtomicUsize,
    pub all_complete: AtomicBool,
}

impl ExploreContext {
    pub fn display(&self, start: Instant) {
        let elapsed = start.elapsed().as_secs();
        let pending = self.pending.load(Ordering::Relaxed);
        let complete = self.complete.load(Ordering::Relaxed);
        let prefix = format!(
            "{:02}:{:02} {} / {}",
            elapsed / 60,
            elapsed % 60,
            complete,
            pending
        );

        let screen_width = terminal_size().map_or(80, |size| (size.0).0 as usize);
        let full_width = screen_width - prefix.len() - 4;
        let set_width = (((full_width * complete) as f32) / (pending as f32)).round() as usize;

        let arrow = if set_width == 0 {
            Cow::Borrowed("")
        } else {
            let mut s = "=".repeat(set_width - 1);
            s.push('>');
            Cow::Owned(s)
        };
        let padding = " ".repeat(full_width - set_width);
        eprint!("{} [{}{}]\r", prefix, arrow, padding);
    }
}

assert_impl_all!(ExploreContext: Send, Sync);

pub async fn explore(dir: PathBuf, shake: u64, ctx: &ExploreContext) -> Node {
    let canon = dir.canonicalize().expect("Could not canonicalize dir");

    let name = pool_rc(
        &ctx.pool,
        canon
            .file_name()
            .map_or(Cow::Borrowed("/"), |str| str.to_string_lossy()),
    );

    let metadata = match fs::symlink_metadata(canon.clone()).await {
        Ok(m) => m,
        Err(err) => return err.into(),
    };

    let size = match metadata_to_size(&dir, &metadata).await {
        Ok(size) => size,
        Err(err) => return err.into(),
    };

    let result = explore_internal(shake, name, canon, size, &ctx).await;
    ctx.all_complete.store(true, Ordering::Relaxed);
    result
}

async fn explore_internal(
    shake: u64,
    name: StringRef,
    dir: PathBuf,
    base_size: Size,
    ctx: &ExploreContext,
) -> Node {
    let mut read = match fs::read_dir(dir).await {
        Ok(read) => read,
        Err(err) => {
            return Node::Error {
                name: None,
                error: err.into(),
            };
        }
    };

    let mut futures = vec![];
    while let Some(entry) = read.next().await {
        let node = entry_to_node(shake, entry, ctx);
        futures.push(node);
    }
    ctx.pending.fetch_add(futures.len(), Ordering::Relaxed);
    let children = join_all(futures).await;

    let mut total = TypedStats::default();
    total.dirs = AggStats {
        count: 1,
        size: base_size,
    };
    let mut by_extension = hashmap![FileTypeExt::Dir => total];

    for child in &children {
        let stat = child.stats();
        total += stat.total;
        for (fte, typed) in stat.by_extension.iter() {
            if by_extension.contains_key(fte) {
                *by_extension.get_mut(fte).unwrap() += *typed;
            } else {
                by_extension.insert(fte.clone(), *typed);
            }
        }
    }

    let stats = NodeStats {
        total,
        by_extension: by_extension.into_iter().collect(),
    };

    Node::Dir {
        name,
        children: children
            .into_iter()
            .filter(|child| child.stats().total.files.size.real >= shake)
            .collect(),
        stats,
    }
}

fn entry_to_node(
    shake: u64,
    entry: io::Result<fs::DirEntry>,
    ctx: &ExploreContext,
) -> BoxFuture<Node> {
    async move {
        let ret = match entry_to_node_res(shake, entry, ctx).await {
            Ok(node) => node,
            Err(err) => err.into(),
        };
        ctx.complete.fetch_add(1, Ordering::Relaxed);
        ret
    }
        .boxed()
}

async fn entry_to_node_res(
    shake: u64,
    entry: io::Result<fs::DirEntry>,
    ctx: &ExploreContext,
) -> io::Result<Node> {
    let entry = match entry {
        Ok(entry) => entry,
        Err(err) => return Err(err),
    };

    let path = entry.path();
    let name = pool_rc_os(&ctx.pool, &entry.file_name());
    let extension = pool_rc(
        &ctx.pool,
        path.extension()
            .map_or(Cow::Borrowed(""), |ext| ext.to_string_lossy()),
    );
    let metadata = entry.metadata().await?;
    let size = metadata_to_size(&path, &metadata).await?;

    let ft = metadata.file_type();

    let ret = if ft.is_dir() {
        explore_internal(shake, name, entry.path(), size, &ctx).await
    } else if ft.is_file() {
        Node::File {
            name,
            extension,
            size,
        }
    } else if ft.is_symlink() {
        Node::Link { name, size }
    } else {
        #[cfg(any(target_os = "unix", target_os = "linux"))]
        {
            use std::os::unix::fs::FileTypeExt;

            if ft.is_block_device() {
                Node::BlockDevice { name, size }
            } else if ft.is_char_device() {
                Node::CharDevice { name, size }
            } else if ft.is_fifo() {
                Node::NamedPipe { name, size }
            } else if ft.is_socket() {
                Node::UnixSocket { name, size }
            } else {
                Node::Other {
                    name,
                    extension,
                    size,
                }
            }
        }

        #[cfg(not(any(target_os = "unix", target_os = "linux")))]
        Node::Other {
            name,
            extension,
            size,
        }
    };

    Ok(ret)
}

async fn metadata_to_size(path: &PathBuf, metadata: &Metadata) -> io::Result<Size> {
    let real = filesize::file_real_size_fast(path, metadata)?;
    let content = metadata.len();
    Ok(Size { real, content })
}

fn pool_rc<S>(string_pool: &StringPool, str: S) -> StringRef
where
    S: AsRef<str>,
{
    let str = str.as_ref();
    let read = string_pool.read().expect("String pool is poisoned");
    if let Some(arc) = read.get(str) {
        return Arc::clone(arc);
    }
    drop(read);

    let arc = Arc::from(str);
    let mut write = string_pool.write().expect("String pool is poisoned");
    write.insert(Arc::clone(&arc));
    drop(write);
    arc
}

fn pool_rc_os(string_pool: &StringPool, oss: &OsStr) -> StringRef {
    pool_rc(string_pool, oss.to_string_lossy())
}
