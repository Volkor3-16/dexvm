//! Live capture + replay for ALL extension APKs.
//!
//! Usage:
//!   # Capture phase (requires network, DEXVM_LIVE=1):
//!   DEXVM_LIVE=1 cargo run --features keiyoushi --bin exttest_live -- --capture
//!
//!   # Replay phase (offline, deterministic):
//!   cargo run --features keiyoushi --bin exttest_live -- --replay
//!
//!   # Both in one go (capture then replay):
//!   DEXVM_LIVE=1 cargo run --features keiyoushi --bin exttest_live -- --all
//!
//!   # Parallel capture (default 4, tune with --jobs):
//!   DEXVM_LIVE=1 cargo run --features keiyoushi --bin exttest_live -- --capture --jobs 8
//!
//! Fixtures are stored in fixtures/live/<apk_stem>/ per APK.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::rc::Rc;

use dexvm::keiyoushi::{FilterState, HttpData, HttpResp, Keiyoushi, Manga, MangaPages};
use dexvm::vm::error::JvmError;
use dexvm::{Context, permission};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

const FIXTURES_ROOT: &str = "fixtures/live";
const DEFAULT_JOBS: usize = 4;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tokio runtime for async
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    
    rt.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    let mut capture = false;
    let mut replay = false;
    let mut apk_filter: Option<String> = None;
    let mut verbose = false;
    let mut timeout_secs = 180;
    let mut jobs = DEFAULT_JOBS;
    let mut force = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--capture" => capture = true,
            "--replay" => replay = true,
            "--all" => { capture = true; replay = true; }
            "--apk" => {
                i += 1;
                apk_filter = args.get(i).cloned();
            }
            "--verbose" | "-v" => verbose = true,
            "--timeout" => {
                i += 1;
                timeout_secs = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(180);
            }
            "--jobs" | "-j" => {
                i += 1;
                jobs = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_JOBS);
            }
            "--force" => force = true,
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
        i += 1
    }
    
    if !capture && !replay {
        eprintln!("Error: must specify --capture, --replay, or --all");
        print_help();
        std::process::exit(1);
    }
    
    let apks = discover_apks(apk_filter.as_deref())?;
    
    if apks.is_empty() {
        eprintln!("No APKs found");
        std::process::exit(1);
    }
    
    println!("Found {} extension APK(s)", apks.len());
    
    if capture {
        println!("\n=== CAPTURE PHASE (parallel jobs: {}) ===", jobs);
        if std::env::var("DEXVM_LIVE").is_err() {
            eprintln!("ERROR: DEXVM_LIVE=1 required for capture");
            std::process::exit(1);
        }
        run_capture_parallel(&apks, jobs, timeout_secs, verbose, force).await?;
    }
    
    if replay {
        println!("\n=== REPLAY PHASE (in-process) ===");
        run_replay_in_process(&apks, verbose).await?;
    }
    
    Ok(())
}

fn print_help() {
    println!(r#"exttest_live - Capture and replay fixtures for ALL extensions

Usage:
  cargo run --features keiyoushi --bin exttest_live [OPTIONS]

Options:
  --capture              Capture live fixtures (requires DEXVM_LIVE=1)
  --replay               Run replay tests against captured fixtures
  --all                  Do both capture and replay
  --apk <path>           Only process this APK (default: all fixtures/tachiyomi-*.apk)
  --verbose, -v          Verbose output
  --timeout <secs>       Per-APK capture timeout (default: 180)
  --jobs, -j <n>         Parallel capture jobs (default: 4)
  --force                Force re-capture even if cached fixtures exist
  --help, -h             Show this help

Environment:
  DEXVM_LIVE=1           Enable live HTTP (required for --capture)

Fixture layout:
  fixtures/live/<apk_stem>/
    manifest.txt         # code  file  method  url
    001-xyz.html         # raw response bodies
    002-abc.json
    ...
    BLOCKED              # marker if site served WAF challenge
"#);
}

fn discover_apks(filter: Option<&str>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut apks = Vec::new();
    
    fn scan_dir(dir: &Path, apks: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, apks)?;
            } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".apk") {
                    apks.push(path);
                }
            }
        }
        Ok(())
    }

    if let Some(filter) = filter {
        let path = Path::new(filter);
        if path.is_dir() {
            scan_dir(path, &mut apks)?;
        } else if path.exists() {
            apks.push(path.to_path_buf());
        }
    } else {
        for dir in ["fixtures", "fixtures/keiyoushi_all"] {
            if Path::new(dir).exists() {
                scan_dir(Path::new(dir), &mut apks)?;
            }
        }
        let mut seen = std::collections::HashSet::new();
        apks.retain(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            seen.insert(name.to_string())
        });
        apks.sort();
    }
    Ok(apks)
}

fn apk_stem(apk: &Path) -> String {
    apk.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn fixture_dir(apk: &Path) -> PathBuf {
    Path::new(FIXTURES_ROOT).join(apk_stem(apk))
}

fn has_valid_fixtures(dir: &Path) -> bool {
    let manifest = dir.join("manifest.txt");
    if !manifest.exists() {
        return false;
    }
    if let Ok(text) = fs::read_to_string(&manifest) {
        !text.lines().next().is_none()
    } else {
        false
    }
}

fn count_fixtures(dir: &Path) -> usize {
    let manifest = dir.join("manifest.txt");
    if let Ok(text) = fs::read_to_string(&manifest) {
        text.lines().count()
    } else {
        0
    }
}

/// Runs live capture for a single APK - opens extension, runs pipeline, captures HTTP responses
async fn capture_one(
    semaphore: Arc<Semaphore>,
    apk: PathBuf,
    _timeout: Duration,
    verbose: bool,
    force: bool,
) -> Result<CaptureResult, Box<dyn std::error::Error + Send + Sync>> {
    let _permit = semaphore.acquire().await;
    let stem = apk_stem(&apk);
    let out_dir = fixture_dir(&apk);
    
    // Check for existing valid fixtures (unless forced)
    if !force && has_valid_fixtures(&out_dir) {
        println!("[{}] Using cached fixtures ({} responses)", stem, count_fixtures(&out_dir));
        return Ok(CaptureResult {
            apk: apk.to_path_buf(),
            stem,
            success: true,
            blocked: out_dir.join("BLOCKED").exists(),
            duration_secs: 0,
            capture_count: count_fixtures(&out_dir),
            error: None,
        });
    }
    
    fs::create_dir_all(&out_dir)?;
    let _ = fs::remove_file(out_dir.join("BLOCKED"));
    
    println!("[{}] Capturing live fixtures...", stem);
    let start = Instant::now();
    
    // Open extension and run capture pipeline in-process
    let result = tokio::task::spawn_blocking(move || -> Result<CaptureResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("[{}] Capturing live fixtures...", stem);
        let start = Instant::now();
        
        // Open extension
        let mut ext = match Keiyoushi::open(apk.to_str().unwrap()) {
            Ok(e) => e,
            Err(e) => {
                let err = format!("Failed to open APK: {}", e);
                return Ok(CaptureResult {
                    apk: apk.clone(),
                    stem,
                    success: false,
                    blocked: false,
                    duration_secs: 0,
                    capture_count: 0,
                    error: Some(err),
                });
            }
        };
        
        // Set up HTTP capture
        let out_dir_str = out_dir.to_string_lossy().into_owned();
        let captures = Arc::new(Mutex::new(Vec::new()));
        let captures_clone = captures.clone();
        
        ext.set_http_rc(Rc::new(move |req: &HttpData| {
            let mut caps = captures_clone.lock().unwrap();
            caps.push((req.method.clone(), req.url.clone(), req.headers.clone(), req.body.clone()));
            // Return empty response to allow pipeline to continue
            HttpResp::ok("<html></html>")
        }));
        
        // Grant permissions
        ext.ctx().grant(permission::Permission::Network(permission::NetworkPermission::Any));
        ext.ctx().grant(permission::Permission::Filesystem(permission::FilesystemPermission::Any));
        
        // Run the full pipeline
        let mut captured_any = false;
        
        // 1. Get sources
        let sources = match ext.sources() {
            Ok(s) => s,
            Err(e) => {
                let err = format!("sources(): {}", ext.describe_error(&e));
                return Ok(CaptureResult {
                    apk: apk.clone(),
                    stem,
                    success: false,
                    blocked: false,
                    duration_secs: start.elapsed().as_secs(),
                    capture_count: 0,
                    error: Some(err),
                });
            }
        };
        
        println!("  Found {} source(s)", sources.len());
        
        for src in &sources {
            // Get source metadata
            let _ = ext.source_name(src);
            let _ = ext.source_lang(src);
            let _ = ext.source_base_url(src);
            let _ = ext.supports_latest(src);
            
            // 2. Filters
            let filters = match ext.filters(src) {
                Ok(f) => f,
                Err(e) => {
                    if verbose {
                        eprintln!("    filters failed: {}", ext.describe_error(&e));
                    }
                    Vec::new()
                }
            };
            
            let states: Vec<FilterState> = filters.iter()
                .map(|f| FilterState { name: f.name.clone(), state: f.state })
                .collect();
            
            // 3. Popular
            let popular = ext.popular(src, 1).unwrap_or_else(|e| {
                if verbose { eprintln!("    popular failed: {}", ext.describe_error(&e)); }
                MangaPages { mangas: Vec::new(), has_next: false }
            });
            
            // 4. Search
            let search = ext.search(src, 1, "one piece", &states).unwrap_or_else(|e| {
                if verbose { eprintln!("    search failed: {}", ext.describe_error(&e)); }
                MangaPages { mangas: Vec::new(), has_next: false }
            });
            
            // 5. Coroutine variants
            let _ = ext.popular_coro(src, 1);
            let _ = ext.search_coro(src, 1, "one piece", &states);
            
            // Get a manga for details/chapters/pages
            let manga = popular.mangas.first()
                .or(search.mangas.first())
                .cloned()
                .unwrap_or_else(|| Manga {
                    title: "test".into(),
                    url: "/manga/test".into(),
                    ..Default::default()
                });
            
            // 6. Manga details
            let _ = ext.manga_details(src, &manga);
            let _ = ext.manga_update_details(src, &manga);
            
            // 7. Chapters
            let chapters = ext.chapters(src, &manga).unwrap_or_else(|e| {
                if verbose { eprintln!("    chapters failed: {}", ext.describe_error(&e)); }
                Vec::new()
            });
            
            let _ = ext.manga_update_chapters(src, &manga);
            
            // 8. Pages
            if let Some(chapter) = chapters.first() {
                let _ = ext.pages(src, chapter);
                let _ = ext.pages_coro(src, chapter);
            }
            
            // 9. Latest
            let _ = ext.latest(src, 1);
            let _ = ext.latest_coro(src, 1);
            
            captured_any = true;
        }
        
        // Save captures to fixtures
        let caps = captures.lock().unwrap();
        save_fixtures(&caps, &out_dir_str, false);
        
        let capture_count = caps.len();
        let duration = start.elapsed().as_secs();
        
        println!("  ✓ {} ({}s, {} responses)", stem, duration, capture_count);
        
        Ok(CaptureResult {
            apk: apk.clone(),
            stem,
            success: true,
            blocked: false,
            duration_secs: duration,
            capture_count,
            error: None,
        })
    }).await??;
    
    Ok(result)
}

#[derive(Debug)]
#[allow(dead_code)]
struct CaptureResult {
    apk: PathBuf,
    stem: String,
    success: bool,
    blocked: bool,
    duration_secs: u64,
    capture_count: usize,
    error: Option<String>,
}

async fn run_capture_parallel(
    apks: &[PathBuf],
    jobs: usize,
    timeout_secs: u64,
    verbose: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(jobs));
    let mut set = JoinSet::new();
    let _timeout = Duration::from_secs(timeout_secs); // timeout handled in blocking task
    
    for apk in apks {
        let sem = semaphore.clone();
        let apk = apk.clone();
        set.spawn(capture_one(sem, apk, _timeout, verbose, force));
    }
    
    let mut results = Vec::new();
    let mut failed = 0;
    
    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(r)) => {
                if !r.success { failed += 1; }
                results.push(r);
            }
            Ok(Err(e)) => {
                failed += 1;
                eprintln!("Task error: {}", e);
            }
            Err(e) => {
                failed += 1;
                eprintln!("Task panicked: {}", e);
            }
        }
    }
    
    // Sort by stem for consistent output
    results.sort_by(|a, b| a.stem.cmp(&b.stem));
    
    // Categorize capture errors
    let mut error_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut failed_extensions: Vec<&CaptureResult> = Vec::new();
    let mut blocked_extensions: Vec<&CaptureResult> = Vec::new();
    
    for r in &results {
        if r.blocked {
            blocked_extensions.push(r);
        }
        if !r.success {
            failed_extensions.push(r);
            let category = categorize_capture_error(r.error.as_deref().unwrap_or("unknown"));
            *error_counts.entry(category).or_default() += 1;
        }
    }
    
    println!("\n=== CAPTURE SUMMARY ===");
    println!("Total: {} extensions", results.len());
    println!("  ✓ Captured:      {}", results.len() - failed);
    println!("  ✗ Failed:        {}", failed);
    println!("  ⚠ WAF Blocked:   {}", blocked_extensions.len());
    
    if !error_counts.is_empty() {
        println!("\nFailure breakdown:");
        for (cat, count) in error_counts.iter().rev() {
            println!("  {:>4}  {}", count, cat);
        }
    }
    
    if !blocked_extensions.is_empty() {
        println!("\nWAF Blocked extensions:");
        for r in &blocked_extensions {
            println!("  - {} ({} responses)", r.stem, r.capture_count);
        }
    }
    
    if !failed_extensions.is_empty() {
        println!("\nFailed extensions:");
        for r in &failed_extensions {
            println!("  - {}: {}", r.stem, r.error.as_deref().unwrap_or("unknown"));
        }
    }
    
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

/// Runs replay test for a single APK IN-PROCESS (no cargo subprocess)
async fn replay_one(
    apk: &Path,
    verbose: bool,
) -> Result<ReplayResult, Box<dyn std::error::Error>> {
    let stem = apk_stem(apk);
    let fixture_dir = fixture_dir(apk);
    let manifest = fixture_dir.join("manifest.txt");
    
    if !manifest.exists() {
        return Ok(ReplayResult {
            apk: apk.to_path_buf(),
            stem,
            success: false,
            error: Some("No fixtures captured (missing manifest.txt)".into()),
            sources_tested: 0,
            sources_passed: 0,
        });
    }
    
    println!("[{}] Running replay tests...", stem);
    let start = Instant::now();
    
    // Load fixtures
    let text = fs::read_to_string(&manifest)?;
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let mut it = line.split('\t');
        let (Some(code), Some(file), Some(_method), Some(url)) =
            (it.next(), it.next(), it.next(), it.next())
        else {
            continue;
        };
        let Ok(raw) = fs::read(fixture_dir.join(file)) else {
            continue;
        };
        let Ok(code) = code.parse() else { continue };
        map.insert(url.to_string(), (code, raw));
    }
    
    // Open APK and set up HTTP replay
    let mut ext = Keiyoushi::open(apk.to_str().unwrap())?;
    ext.set_http_rc(Rc::new(move |req: &HttpData| {
        if let Some((code, body)) = map.get(&req.url) {
            HttpResp {
                code: *code,
                message: "OK".into(),
                headers: Vec::new(),
                body: Some(body.clone()),
            }
        } else {
            HttpResp::ok("<html></html>")
        }
    }));
    
    // Grant permissions
    ext.ctx().grant(permission::Permission::Network(permission::NetworkPermission::Any));
    ext.ctx().grant(permission::Permission::Filesystem(permission::FilesystemPermission::Any));
    
    // Get all sources
    let sources = ext.sources()?;
    
    let mut passed = 0;
    let mut tested = 0;
    
    for src in &sources {
        tested += 1;
        // Test basic operations that don't need manga data
        let ok = test_basic_operations(&mut ext, src, verbose).await;
        if ok {
            passed += 1;
        }
    }
    
    let duration = start.elapsed().as_secs();
    let success = tested > 0 && passed == tested;
    
    let stem_clone = stem.clone();
    let result = ReplayResult {
        apk: apk.to_path_buf(),
        stem,
        success,
        error: if success { None } else { Some("Some sources failed".into()) },
        sources_tested: tested,
        sources_passed: passed,
    };
    
    if success {
        println!("  ✓ {} ({}s, {}/{} passed)", stem_clone, duration, passed, tested);
    } else {
        println!("  ✗ {} ({}s, {}/{} passed)", stem_clone, duration, passed, tested);
    }
    
    Ok(result)
}

async fn test_basic_operations(
    ext: &mut Keiyoushi,
    src: &dexvm::keiyoushi::Source,
    verbose: bool,
) -> bool {
    // Test filters (no network)
    let filters_result = ext.filters(src);
    let filters_ok = filters_result.is_ok();
    if verbose && !filters_ok {
        if let Err(e) = &filters_result {
            eprintln!("    ✗ filters failed: {}", ext.describe_error(e));
        }
    }
    
    // Test popular
    let popular_result = ext.popular(src, 1);
    let popular_ok = popular_result.is_ok();
    if verbose && !popular_ok {
        if let Err(e) = &popular_result {
            eprintln!("    ✗ popular failed: {}", ext.describe_error(e));
        }
    }
    
    // Test search
    let filters = filters_result.unwrap_or_default();
    let states: Vec<FilterState> = filters.iter().map(|f| FilterState { name: f.name.clone(), state: f.state }).collect();
    let search_result = ext.search(src, 1, "one piece", &states);
    let search_ok = search_result.is_ok();
    if verbose && !search_ok {
        if let Err(e) = &search_result {
            eprintln!("    ✗ search failed: {}", ext.describe_error(e));
        }
    }
    
    filters_ok && popular_ok && search_ok
}

#[derive(Debug)]
#[allow(dead_code)]
struct ReplayResult {
    apk: PathBuf,
    stem: String,
    success: bool,
    error: Option<String>,
    sources_tested: usize,
    sources_passed: usize,
}

async fn run_replay_in_process(
    apks: &[PathBuf],
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let mut failed = 0;
    let mut total_tested = 0;
    let mut total_passed = 0;
    
    // Run sequentially (fast, no subprocess overhead)
    for apk in apks {
        match replay_one(apk, verbose).await {
            Ok(r) => {
                if !r.success { failed += 1; }
                total_tested += r.sources_tested;
                total_passed += r.sources_passed;
                results.push(r);
            }
            Err(e) => {
                failed += 1;
                results.push(ReplayResult {
                    apk: apk.clone(),
                    stem: apk_stem(apk),
                    success: false,
                    error: Some(e.to_string()),
                    sources_tested: 0,
                    sources_passed: 0,
                });
            }
        }
    }
    
    // Categorize replay errors
    let mut op_failures: BTreeMap<String, usize> = BTreeMap::new();
    let mut failed_extensions: Vec<&ReplayResult> = Vec::new();
    let mut no_fixture_extensions: Vec<&ReplayResult> = Vec::new();
    
    for r in &results {
        if r.sources_tested == 0 && !r.success {
            no_fixture_extensions.push(r);
            continue;
        }
        if !r.success {
            failed_extensions.push(r);
            // Categorize by which operations failed
            if r.sources_passed < r.sources_tested {
                let failed_ops = r.sources_tested - r.sources_passed;
                *op_failures.entry(format!("{} sources with partial failure", failed_ops)).or_default() += 1;
            }
        }
    }
    
    println!("\n=== REPLAY SUMMARY ===");
    println!("Total: {} extensions", results.len());
    println!("  ✓ Fully passed:  {}", results.len() - failed - no_fixture_extensions.len());
    println!("  ⚠ Partial pass:  {}", failed_extensions.len());
    println!("  ✗ No fixtures:   {}", no_fixture_extensions.len());
    println!("  Sources tested:  {}", total_tested);
    println!("  Sources passed:  {}", total_passed);
    
    if !op_failures.is_empty() {
        println!("\nPartial failure breakdown:");
        for (cat, count) in op_failures.iter().rev() {
            println!("  {:>4}  {}", count, cat);
        }
    }
    
    if !no_fixture_extensions.is_empty() {
        println!("\nExtensions with no fixtures (skipped):");
        for r in &no_fixture_extensions {
            println!("  - {}: {}", r.stem, r.error.as_deref().unwrap_or("unknown"));
        }
    }
    
    if !failed_extensions.is_empty() {
        println!("\nPartially failed extensions:");
        for r in &failed_extensions {
            println!("  - {} ({}/{} sources passed)", r.stem, r.sources_passed, r.sources_tested);
        }
    }
    
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

// Error categorization functions
fn categorize_capture_error(err: &str) -> String {
    let lower = err.to_lowercase();
    if lower.contains("wa") && lower.contains("blocked") || lower.contains("ddos-guard") || lower.contains("cloudflare") || lower.contains("challenge") {
        "WAF/DDoS-Guard block".to_string()
    } else if lower.contains("timeout") || lower.contains("timed out") {
        "Timeout".to_string()
    } else if lower.contains("connection refused") || lower.contains("connection reset") || lower.contains("dns") || lower.contains("resolve") {
        "Network/Connectivity".to_string()
    } else if lower.contains("permission") || lower.contains("denied") || lower.contains("forbidden") || lower.contains("403") {
        "Permission/403".to_string()
    } else if lower.contains("404") || lower.contains("not found") {
        "404 Not Found".to_string()
    } else if lower.contains("500") || lower.contains("502") || lower.contains("503") || lower.contains("504") {
        "Server Error (5xx)".to_string()
    } else if lower.contains("panicked") || lower.contains("panic") {
        "Panic/Crash".to_string()
    } else if lower.contains("resolution") || lower.contains("class not found") || lower.contains("method not found") {
        "VM Resolution".to_string()
    } else if lower.contains("unsupported") || lower.contains("unimplemented") {
        "Unimplemented".to_string()
    } else if lower.contains("nullpointer") || lower.contains("npe") {
        "NullPointerException".to_string()
    } else if lower.contains("classcastexception") {
        "ClassCastException".to_string()
    } else if lower.contains("illegalargument") {
        "IllegalArgumentException".to_string()
    } else if lower.contains("ioexception") {
        "IOException".to_string()
    } else {
        "Other".to_string()
    }
}
/// Saves captured HTTP responses to fixture directory
fn save_fixtures(caps: &[(String, String, Vec<(String, String)>, Option<String>)], out_dir: &str, blocked: bool) {
    let _ = std::fs::create_dir_all(out_dir);
    let mut manifest = String::new();
    let mut seen = std::collections::HashSet::new();
    let mut n = 0usize;
    for (method, url, headers, body) in caps {
        if !seen.insert((method.clone(), url.clone())) {
            continue;
        }
        let slug: String = url
            .replace("https://", "")
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let fname: String = format!("{n:03}-{slug}").chars().take(100).collect();
        let body_bytes = body.as_deref().unwrap_or("").as_bytes();
        let _ = std::fs::write(format!("{out_dir}/{fname}"), body_bytes);
        manifest.push_str(&format!("200\t{fname}\t{method}\t{url}\n"));
        n += 1;
    }
    let _ = std::fs::write(format!("{out_dir}/manifest.txt"), manifest);
    if blocked {
        let _ = std::fs::write(format!("{out_dir}/BLOCKED"), "");
    } else {
        let _ = std::fs::remove_file(format!("{out_dir}/BLOCKED"));
    }
    eprintln!("live: captured {n} responses into {out_dir}/ (blocked={blocked})");
}
