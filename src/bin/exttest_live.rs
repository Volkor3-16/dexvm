// Live capture + replay for ALL extension APKs.
//!
//! Usage:
//!   # Capture phase (requires network, DEXVM_LIVE=1):
//!   DEXVM_LIVE=1 cargo run --features keiyoushi,live --bin exttest_live -- --capture
//!
//!   # Replay phase (offline, deterministic):
//!   cargo run --features keiyoushi,live --bin exttest_live -- --replay
//!
//!   # Both in one go (capture then replay):
//!   DEXVM_LIVE=1 cargo run --features keiyoushi,live --bin exttest_live -- --all
//!
//!   # Parallel capture (default 4, tune with --jobs):
//!   DEXVM_LIVE=1 cargo run --features keiyoushi,live --bin exttest_live -- --capture --jobs 8
//!
//! Fixtures are stored in fixtures/live/<apk_stem>/ per APK.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dexvm::keiyoushi::{FilterState, HttpData, HttpResp, Keiyoushi, Manga, MangaPages};
use dexvm::vm::error::JvmError;
use dexvm::{permission, Context, ContextError};

#[cfg(feature = "live")]
use tokio::sync::Semaphore;
#[cfg(feature = "live")]
use tokio::task::JoinSet;

const FIXTURES_ROOT: &str = "fixtures/live";
const DEFAULT_JOBS: usize = 4;

#[cfg(feature = "live")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tokio runtime for async
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async_main())
}

#[cfg(feature = "live")]
async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut capture = false;
    let mut replay = false;
    let mut apk_filter: Option<String> = None;
    let mut verbose = false;
    let mut timeout_secs = 180;
    let mut jobs = DEFAULT_JOBS;

    for arg in args {
        match arg.as_str() {
            "--capture" => capture = true,
            "--replay" => replay = true,
            "--all" => { capture = true; replay = true; }
            "--verbose" => verbose = true,
            "--jobs" => {
                // next arg is jobs
            }
            _ => {
                if arg.starts_with("--jobs=") {
                    jobs = arg[7..].parse().unwrap_or(DEFAULT_JOBS);
                } else if !arg.starts_with("-") {
                    apk_filter = Some(arg);
                }
            }
        }
    }

    if !capture && !replay {
        eprintln!("Usage: exttest_live [--capture|--replay|--all] [--jobs N] [--apk PATH]");
        return Ok(());
    }

    if capture {
        eprintln!("Capture mode not fully implemented in this version");
    }
    if replay {
        eprintln!("Replay mode not fully implemented in this version");
    }

    Ok(())
}

#[cfg(not(feature = "live"))]
fn main() {
    eprintln!("exttest_live requires the 'live' feature to be enabled");
    eprintln!("Run with: cargo run --features keiyoushi,live --bin exttest_live");
    std::process::exit(1);
}
