//! Extension tester: runs the full keiyoushi pipeline against ALL extension
//! APKs in fixtures/ and ALL sources within each extension. Produces a
//! structured JSON report + human summary for regression tracking.
//!
//! Usage:
//!   cargo run --features keiyoushi --bin exttest
//!   cargo run --features keiyoushi --bin exttest -- --json report.json
//!   cargo run --features keiyoushi --bin exttest -- --apk fixtures/tachiyomi-all.akuma-v1.4.10.apk

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use dexvm::keiyoushi::{FilterState, HttpData, HttpResp, Keiyoushi, Manga, MangaPages};
use dexvm::vm::error::JvmError;
use dexvm::{permission, Context};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceResult {
    name: String,
    lang: String,
    base_url: String,
    source_id: i64,
    supports_latest: bool,
    operations: BTreeMap<String, OperationResult>,
    filters: Vec<FilterResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FilterResult {
    name: String,
    kind: String,
    state: i32,
    options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperationResult {
    success: bool,
    duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtensionResult {
    apk_path: String,
    apk_name: String,
    manifest_package: Option<String>,
    manifest_name: Option<String>,
    manifest_version: Option<String>,
    sources: Vec<SourceResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_error: Option<String>,
    total_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestReport {
    timestamp: String,
    git_commit: Option<String>,
    git_branch: Option<String>,
    extensions: Vec<ExtensionResult>,
    summary: Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Summary {
    total_extensions: usize,
    total_sources: usize,
    sources_by_status: BTreeMap<String, usize>,
    operations_by_status: BTreeMap<String, usize>,
    failed_sources: Vec<FailedSourceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FailedSourceSummary {
    extension: String,
    source: String,
    error_type: String,
    error_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ErrorType {
    VmInit,
    SourceEnumeration,
    SourceMetadata,
    Filters,
    Popular,
    PopularCoro,
    Search,
    SearchCoro,
    MangaDetails,
    MangaUpdateDetails,
    Chapters,
    MangaUpdateChapters,
    Pages,
    PagesCoro,
    Latest,
    LatestCoro,
    ImageData,
    HttpExecution,
    Permission,
    Unknown,
}

impl ErrorType {
    fn as_str(&self) -> &'static str {
        match self {
            ErrorType::VmInit => "vm_init",
            ErrorType::SourceEnumeration => "source_enumeration",
            ErrorType::SourceMetadata => "source_metadata",
            ErrorType::Filters => "filters",
            ErrorType::Popular => "popular",
            ErrorType::PopularCoro => "popular_coro",
            ErrorType::Search => "search",
            ErrorType::SearchCoro => "search_coro",
            ErrorType::MangaDetails => "manga_details",
            ErrorType::MangaUpdateDetails => "manga_update_details",
            ErrorType::Chapters => "chapters",
            ErrorType::MangaUpdateChapters => "manga_update_chapters",
            ErrorType::Pages => "pages",
            ErrorType::PagesCoro => "pages_coro",
            ErrorType::Latest => "latest",
            ErrorType::LatestCoro => "latest_coro",
            ErrorType::ImageData => "image_data",
            ErrorType::HttpExecution => "http_execution",
            ErrorType::Permission => "permission",
            ErrorType::Unknown => "unknown",
        }
    }
}

struct TestConfig {
    apk_filter: Option<String>,
    json_output: Option<String>,
    compare: Option<String>,
    verbose: bool,
    timeout_secs: u64,
    http_mode: HttpMode,
    require_fixtures: bool,
    fresh_vm_per_source: bool,
}

#[derive(Debug, Clone, Copy)]
enum HttpMode {
    /// Replay from fixtures/live/ (offline deterministic)
    Replay,
    /// Use empty responses (fast, tests code paths only)
    Empty,
    /// Use real HTTP (requires DEXVM_LIVE=1)
    Live,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut config = TestConfig {
        apk_filter: None,
        json_output: None,
        compare: None,
        verbose: false,
        timeout_secs: 120,
        http_mode: HttpMode::Replay,
        require_fixtures: false,
        fresh_vm_per_source: false,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--apk" => {
                i += 1;
                config.apk_filter = args.get(i).cloned();
            }
            "--json" => {
                i += 1;
                config.json_output = args.get(i).cloned();
            }
            "--compare" => {
                i += 1;
                config.compare = args.get(i).cloned();
            }
            "--verbose" | "-v" => config.verbose = true,
            "--timeout" => {
                i += 1;
                config.timeout_secs = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(120);
            }
            "--http" => {
                i += 1;
                config.http_mode = match args.get(i).map(|s| s.as_str()) {
                    Some("replay") => HttpMode::Replay,
                    Some("empty") => HttpMode::Empty,
                    Some("live") => HttpMode::Live,
                    _ => HttpMode::Replay,
                };
            }
            "--require-fixtures" => {
                config.require_fixtures = true;
            }
            "--shared-vm" => {
                config.fresh_vm_per_source = false;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
        i += 1
    }

    let report = run_tests(&config)?;
    output_report(&report, &config)?;

    // Handle compare mode (after running current tests)
    if let Some(compare_path) = &config.compare {
        return compare_reports(compare_path, &report, config.json_output.as_deref());
    }

    // Exit with error code if any sources failed
    if report.summary.failed_sources.is_empty() {
        println!("\n✓ All sources passed");
        Ok(())
    } else {
        eprintln!(
            "\n✗ {} source(s) failed",
            report.summary.failed_sources.len()
        );
        std::process::exit(1);
    }
}

fn print_help() {
    println!(
        r#"exttest - Extension regression tester for dexvm

Usage:
  cargo run --features keiyoushi --bin exttest [OPTIONS]

Options:
  --apk <path>              Test only this APK (default: all fixtures/tachiyomi-*.apk)
  --json <path>             Write JSON report to file
  --compare <path>          Compare with a previous JSON report
  --verbose, -v             Verbose output
  --timeout <secs>          Per-operation timeout (default: 120)
  --http <mode>             HTTP mode: replay|empty|live (default: replay)
  --require-fixtures        Fail if fixtures/live/ missing in replay mode
  --shared-vm               Reuse single VM across all sources (default: fresh per source)
  --help, -h                Show this help

HTTP modes:
  replay   - Replay from fixtures/live/ (deterministic, offline)
  empty    - Return empty JSON/HTML for all requests (fast, tests VM code paths)
  live     - Real HTTP to source sites (requires DEXVM_LIVE=1)

Environment:
  DEXVM_LIVE=1              Enable live HTTP mode
  RUST_LOG=info             Enable VM debug traces (DBG/ERR/INV)
"#
    );
}

fn run_tests(config: &TestConfig) -> Result<TestReport, Box<dyn std::error::Error>> {
    let _start_time = Instant::now();
    let apks = discover_apks(config.apk_filter.as_deref())?;

    if apks.is_empty() {
        eprintln!("No APKs found in fixtures/");
        std::process::exit(1);
    }

    println!("Found {} extension APK(s)", apks.len());
    for apk in &apks {
        println!("  - {}", apk);
    }

    let mut extensions = Vec::new();
    for apk_path in &apks {
        let ext_result = test_extension(apk_path, config)?;
        extensions.push(ext_result);
    }

    let summary = build_summary(&extensions);
    let report = TestReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        git_commit: get_git_commit(),
        git_branch: get_git_branch(),
        extensions,
        summary,
    };

    Ok(report)
}

fn discover_apks(filter: Option<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut apks = Vec::new();

    fn scan_dir(dir: &Path, apks: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, apks)?;
            } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".apk") {
                    apks.push(path.to_string_lossy().into_owned());
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
            apks.push(filter.to_string());
        } else {
            eprintln!("APK not found: {}", filter);
            std::process::exit(1);
        }
    } else {
        // Default: scan fixtures/ and fixtures/keiyoushi_all/ if it exists
        for dir in ["fixtures", "fixtures/keiyoushi_all"] {
            if Path::new(dir).exists() {
                scan_dir(Path::new(dir), &mut apks)?;
            }
        }
        // Deduplicate by filename (keep first occurrence)
        let mut seen = std::collections::HashSet::new();
        apks.retain(|p| {
            let name = Path::new(p)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            seen.insert(name.to_string())
        });
        apks.sort();
    }
    Ok(apks)
}

fn test_extension(
    apk_path: &str,
    config: &TestConfig,
) -> Result<ExtensionResult, Box<dyn std::error::Error>> {
    let apk_name = Path::new(apk_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(apk_path)
        .to_string();

    let ext_start = Instant::now();
    println!("\n=== Testing {} ===", apk_name);

    // Load manifest for metadata
    let (manifest_package, manifest_name, manifest_version) =
        load_manifest(apk_path).unwrap_or((None, None, None));

    // Create Keiyoushi with HTTP handler
    let mut ext = match Keiyoushi::open(apk_path) {
        Ok(ext) => ext,
        Err(e) => {
            let err = format!("Failed to load APK: {}", e);
            eprintln!("  ✗ {}", err);
            return Ok(ExtensionResult {
                apk_path: apk_path.to_string(),
                apk_name,
                manifest_package,
                manifest_name,
                manifest_version,
                sources: Vec::new(),
                load_error: Some(err),
                total_duration_ms: ext_start.elapsed().as_millis() as u64,
            });
        }
    };

    // Set up HTTP callback based on mode
    setup_http(&mut ext, config.http_mode, config.require_fixtures);

    // Grant permissions
    ext.ctx().grant(permission::Permission::Network(
        permission::NetworkPermission::Any,
    ));
    ext.ctx().grant(permission::Permission::Filesystem(
        permission::FilesystemPermission::Any,
    ));

    // Get all sources
    let sources = match ext.sources() {
        Ok(s) => s,
        Err(e) => {
            let err = format!("sources(): {}", ext.describe_error(&e));
            eprintln!("  ✗ {}", err);
            return Ok(ExtensionResult {
                apk_path: apk_path.to_string(),
                apk_name,
                manifest_package,
                manifest_name,
                manifest_version,
                sources: Vec::new(),
                load_error: Some(err),
                total_duration_ms: ext_start.elapsed().as_millis() as u64,
            });
        }
    };

    println!("  Found {} source(s)", sources.len());

    let mut source_results = Vec::new();
    for src in &sources {
        let src_result = if config.fresh_vm_per_source {
            // Create fresh VM per source
            let mut new_ext = Keiyoushi::open(apk_path)?;
            setup_http(&mut new_ext, config.http_mode, config.require_fixtures);
            new_ext.ctx().grant(permission::Permission::Network(
                permission::NetworkPermission::Any,
            ));
            new_ext.ctx().grant(permission::Permission::Filesystem(
                permission::FilesystemPermission::Any,
            ));
            test_source(&mut new_ext, src, config)?
        } else {
            // Reuse the same ext instance (pass reference)
            test_source(&mut ext, src, config)?
        };
        source_results.push(src_result);
    }

    Ok(ExtensionResult {
        apk_path: apk_path.to_string(),
        apk_name,
        manifest_package,
        manifest_name,
        manifest_version,
        sources: source_results,
        load_error: None,
        total_duration_ms: ext_start.elapsed().as_millis() as u64,
    })
}

fn load_manifest(
    apk_path: &str,
) -> Result<(Option<String>, Option<String>, Option<String>), Box<dyn std::error::Error>> {
    let mut ctx = Context::open(apk_path)?;
    let manifest = ctx.manifest()?;
    Ok((
        Some(manifest.package_id),
        Some(manifest.app_name),
        manifest.version_name,
    ))
}

fn setup_http(ext: &mut Keiyoushi, mode: HttpMode, require_fixtures: bool) {
    match mode {
        HttpMode::Replay => {
            // Support custom fixture directory via DEXVM_LIVE_DIR
            let live_dir = std::env::var("DEXVM_LIVE_DIR").unwrap_or_else(|_| "fixtures/live".to_string());
            let manifest_path = format!("{live_dir}/manifest.txt");
            if Path::new(&manifest_path).exists() {
                let text = fs::read_to_string(&manifest_path).unwrap_or_default();
                let mut map = HashMap::new();
                for line in text.lines() {
                    let mut it = line.split('\t');
                    let (Some(code), Some(file), Some(_method), Some(url)) =
                        (it.next(), it.next(), it.next(), it.next())
                    else {
                        continue;
                    };
                    let Ok(raw) = fs::read(format!("{live_dir}/{file}")) else {
                        continue;
                    };
                    let Ok(code) = code.parse() else { continue };
                    map.insert(url.to_string(), (code, raw));
                }
                ext.set_http_rc(std::rc::Rc::new(move |req: &HttpData| {
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
            } else if require_fixtures {
                eprintln!("  ✗ {manifest_path} not found (use --require-fixtures to make this an error)");
                std::process::exit(1);
            } else {
                // No fixtures, fall back to empty
                eprintln!("  ⚠ {manifest_path} not found, falling back to empty HTTP mode");
                ext.set_http(|_| HttpResp::ok("<html></html>"));
            }
        }
        HttpMode::Empty => {
            // Return minimal valid JSON responses that satisfy common API expectations:
            // - GraphQL: {"data":{"popularManga":{"mangas":[],"hasNext":false},...}}
            // - REST MangasPage: {"mangas":[],"hasNext":false}
            // - REST Chapter/Page lists: []
            // This avoids JSON parsing errors and "missing data field" errors.
            ext.set_http(|req: &HttpData| {
                let url = req.url.to_lowercase();
                // Extract path only (before ?) for endpoint detection
                let path = url.split('?').next().unwrap_or(&url);
                // Check for GraphQL via Accept header or URL
                let is_graphql = url.contains("graphql")
                    || req.headers.iter().any(|(k, v)| k.to_lowercase() == "accept" && v.contains("graphql"));
                let body = if is_graphql {
                    // GraphQL API - return empty but valid responses for common queries
                    if url.contains("popular") {
                        r#"{"data":{"popularManga":{"mangas":[],"hasNext":false}}}"#.to_string()
                    } else if url.contains("search") {
                        r#"{"data":{"searchManga":{"mangas":[],"hasNext":false}}}"#.to_string()
                    } else if url.contains("latest") || url.contains("update") {
                        r#"{"data":{"latestUpdates":{"mangas":[],"hasNext":false}}}"#.to_string()
                    } else if url.contains("chapter") || url.contains("page") {
                        r#"{"data":{"chapterList":[]}}"#.to_string()
                    } else if url.contains("detail") || url.contains("info") || url.contains("manga") {
                        r#"{"data":{"manga":{"title":"","url":"","description":"","author":"","artist":"","genre":"","status":0,"thumbnailUrl":"","coverUrl":""}}}"#.to_string()
                    } else {
                        // Generic empty GraphQL response
                        r#"{"data":{}}"#.to_string()
                    }
                } else if path.contains("popular") || path.contains("search") || path.contains("latest") || path.contains("update") || path.contains("list") {
                    // Likely a MangasPage or chapter list endpoint
                    if path.contains("chapter") || path.contains("page") {
                        r#"[]"#.to_string()
                    } else {
                        r#"{"mangas":[],"hasNext":false}"#.to_string()
                    }
                } else if path.contains("detail") || path.contains("info") || path.contains("manga") {
                    // Manga details endpoint
                    r#"{"title":"","url":"","description":"","author":"","artist":"","genre":"","status":0,"thumbnailUrl":"","coverUrl":""}"#.to_string()
                } else if path.contains("chapter") || path.contains("page") {
                    // Chapter or page list
                    r#"[]"#.to_string()
                } else {
                    // Default: minimal valid JSON object
                    "{}".to_string()
                };
                HttpResp::ok(body)
            });
        }
        HttpMode::Live => {
            // Use real HTTP (requires ureq)
            ext.set_http(move |req: &HttpData| {
                static AGENT: std::sync::OnceLock<ureq::Agent> = std::sync::OnceLock::new();
                let agent = AGENT.get_or_init(|| {
                    ureq::Agent::config_builder()
                        .user_agent(
                            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
                             Chrome/126.0 Safari/537.36",
                        )
                        .timeout_global(Some(std::time::Duration::from_secs(30)))
                        .http_status_as_error(false)
                        .build()
                        .into()
                });
                if req.method == "POST" {
                    let mut rq = agent.post(&req.url);
                    for (k, v) in &req.headers {
                        rq = rq.header(k, v);
                    }
                    match rq.send(req.body.as_deref().unwrap_or("")) {
                        Ok(r) => {
                            let code = r.status().as_u16() as i32;
                            let bytes = r.into_body().read_to_vec().unwrap_or_default();
                            HttpResp { code, message: "OK".into(), headers: Vec::new(), body: Some(bytes) }
                        }
                        Err(e) => HttpResp {
                            code: 0,
                            message: e.to_string(),
                            headers: Vec::new(),
                            body: None,
                        },
                    }
                } else {
                    let mut rq = agent.get(&req.url);
                    for (k, v) in &req.headers {
                        rq = rq.header(k, v);
                    }
                    match rq.call() {
                        Ok(r) => {
                            let code = r.status().as_u16() as i32;
                            let bytes = r.into_body().read_to_vec().unwrap_or_default();
                            HttpResp { code, message: "OK".into(), headers: Vec::new(), body: Some(bytes) }
                        }
                        Err(e) => HttpResp {
                            code: 0,
                            message: e.to_string(),
                            headers: Vec::new(),
                            body: None,
                        },
                    }
                }
            });
        }
    }
}

fn test_source(
    ext: &mut Keiyoushi,
    src: &dexvm::keiyoushi::Source,
    config: &TestConfig,
) -> Result<SourceResult, Box<dyn std::error::Error>> {
    let _source_start = Instant::now();
    let mut operations = BTreeMap::new();
    let mut source_error: Option<String> = None;

    // Get source metadata - fail if any of these fail
    let name = match ext.source_name(src) {
        Ok(n) => n,
        Err(e) => {
            let err = format!("source_name failed: {}", ext.describe_error(&e));
            if config.verbose {
                eprintln!("    ✗ source_name: {}", err);
            }
            operations.insert(
                "source_name".to_string(),
                OperationResult {
                    success: false,
                    duration_ms: 0,
                    data_summary: None,
                    error: Some(err.clone()),
                    error_type: Some(classify_error_str(&err).as_str().to_string()),
                },
            );
            source_error = Some(err);
            "unknown".to_string()
        }
    };
    let lang = match ext.source_lang(src) {
        Ok(l) => l,
        Err(e) => {
            let err = format!("source_lang failed: {}", ext.describe_error(&e));
            if config.verbose {
                eprintln!("    ✗ source_lang: {}", err);
            }
            operations.insert(
                "source_lang".to_string(),
                OperationResult {
                    success: false,
                    duration_ms: 0,
                    data_summary: None,
                    error: Some(err.clone()),
                    error_type: Some(classify_error_str(&err).as_str().to_string()),
                },
            );
            if source_error.is_none() {
                source_error = Some(err.clone());
            }
            "unknown".to_string()
        }
    };
    let base_url = ext.source_base_url(src).unwrap_or_else(|_| "".to_string());
    let source_id = ext.source_id(src).unwrap_or(-1);
    let supports_latest = ext.supports_latest(src).unwrap_or(false);

    if config.verbose {
        println!("  Source: {} ({})", name, lang);
    }

    // Test filters
    let filters = test_operation(
        ext,
        src,
        "filters",
        ErrorType::Filters,
        config,
        &mut operations,
        |ext, src| {
            let fl = ext.filters(src)?;
            Ok(fl)
        },
    );
    if filters.is_err() {
        source_error = Some("filters failed".to_string());
    }

    // Test popular
    let popular_result = test_operation(
        ext,
        src,
        "popular",
        ErrorType::Popular,
        config,
        &mut operations,
        |ext, src| {
            let pages = ext.popular(src, 1)?;
            Ok(pages)
        },
    );
    if popular_result.is_err() {
        if source_error.is_none() {
            source_error = Some("popular failed".to_string());
        }
    }

    // Test search (using default filter states from filters)
    let filters_for_search = ext.filters(src).unwrap_or_default();
    let filter_states: Vec<FilterState> = filters_for_search
        .iter()
        .map(|f| FilterState {
            name: f.name.clone(),
            state: f.state,
        })
        .collect();

    let search_result = test_operation(
        ext,
        src,
        "search",
        ErrorType::Search,
        config,
        &mut operations,
        |ext, src| {
            let pages = ext.search(src, 1, "one piece", &filter_states)?;
            Ok(pages)
        },
    );
    if search_result.is_err() {
        if source_error.is_none() {
            source_error = Some("search failed".to_string());
        }
    }

    // Test coroutine variants (these don't require a manga from regular tests)
    // Test popular_coro
    let popular_coro_result = test_operation(
        ext,
        src,
        "popular_coro",
        ErrorType::PopularCoro,
        config,
        &mut operations,
        |ext, src| {
            let pages = ext.popular_coro(src, 1)?;
            Ok(pages)
        },
    );

    // Test search_coro
    let search_coro_result = test_operation(
        ext,
        src,
        "search_coro",
        ErrorType::SearchCoro,
        config,
        &mut operations,
        |ext, src| {
            let pages = ext.search_coro(src, 1, "one piece", &filter_states)?;
            Ok(pages)
        },
    );

    // Test latest_coro (if supported)
    if supports_latest {
        let _ = test_operation(
            ext,
            src,
            "latest_coro",
            ErrorType::LatestCoro,
            config,
            &mut operations,
            |ext, src| {
                let pages = ext.latest_coro(src, 1)?;
                Ok(pages)
            },
        );
    } else {
        mark_skipped(
            &mut operations,
            "latest_coro",
            ErrorType::LatestCoro,
            "not supported by source",
        );
    }

    // Get a manga for details/chapters/pages tests (try regular first, then coroutine)
    let manga = match get_test_manga(&popular_result, &search_result) {
        Some(m) => m,
        None => match get_test_manga(&popular_coro_result, &search_coro_result) {
            Some(m) => m,
            None => {
                // No manga available from either regular or coroutine tests
                mark_skipped(
                    &mut operations,
                    "manga_details",
                    ErrorType::MangaDetails,
                    "no manga to test",
                );
                mark_skipped(
                    &mut operations,
                    "chapters",
                    ErrorType::Chapters,
                    "no manga to test",
                );
                mark_skipped(
                    &mut operations,
                    "pages",
                    ErrorType::Pages,
                    "no manga to test",
                );
                mark_skipped(
                    &mut operations,
                    "manga_update_details",
                    ErrorType::MangaUpdateDetails,
                    "no manga to test",
                );
                mark_skipped(
                    &mut operations,
                    "manga_update_chapters",
                    ErrorType::MangaUpdateChapters,
                    "no manga to test",
                );
                mark_skipped(
                    &mut operations,
                    "pages_coro",
                    ErrorType::PagesCoro,
                    "no manga to test",
                );
                if config.verbose {
                    println!("    No manga found, skipping details/chapters/pages");
                }
                return Ok(SourceResult {
                    name,
                    lang,
                    base_url,
                    source_id,
                    supports_latest,
                    operations,
                    filters: filters_for_search
                        .into_iter()
                        .map(|f| FilterResult {
                            name: f.name,
                            kind: format!("{:?}", f.kind),
                            state: f.state,
                            options: f.options,
                        })
                        .collect(),
                    error: source_error,
                });
            }
        },
    };

    // Test manga_details
    let details_result = test_operation(
        ext,
        src,
        "manga_details",
        ErrorType::MangaDetails,
        config,
        &mut operations,
        |ext, src| {
            let details = ext.manga_details(src, &manga)?;
            Ok(details)
        },
    );
    if details_result.is_err() {
        if source_error.is_none() {
            source_error = Some("manga_details failed".to_string());
        }
    }

    // Test chapters
    let chapters_result = test_operation(
        ext,
        src,
        "chapters",
        ErrorType::Chapters,
        config,
        &mut operations,
        |ext, src| {
            let chapters = ext.chapters(src, &manga)?;
            Ok(chapters)
        },
    );
    if chapters_result.is_err() {
        if source_error.is_none() {
            source_error = Some("chapters failed".to_string());
        }
    }

    // Test pages (using first chapter)
    let pages_result = test_operation(
        ext,
        src,
        "pages",
        ErrorType::Pages,
        config,
        &mut operations,
        |ext, src| {
            let chapters = ext.chapters(src, &manga)?;
            if let Some(chapter) = chapters.first() {
                let pages = ext.pages(src, chapter)?;
                Ok(pages)
            } else {
                Err(dexvm::vm::error::JvmError::Resolution("no chapters".into()))
            }
        },
    );
    if pages_result.is_err() {
        if source_error.is_none() {
            source_error = Some("pages failed".to_string());
        }
    }

    // Test latest (if supported)
    if supports_latest {
        let latest_result = test_operation(
            ext,
            src,
            "latest",
            ErrorType::Latest,
            config,
            &mut operations,
            |ext, src| {
                let pages = ext.latest(src, 1)?;
                Ok(pages)
            },
        );
        if latest_result.is_err() {
            if source_error.is_none() {
                source_error = Some("latest failed".to_string());
            }
        }
    } else {
        mark_skipped(
            &mut operations,
            "latest",
            ErrorType::Latest,
            "not supported by source",
        );
    }

    // Test image_data (if we have pages, or use a known test URL)
    let test_image_url = if let Ok(pages_vec) = &pages_result {
        pages_vec.first().map(|p| p.image_url.clone())
    } else {
        None
    }
    .unwrap_or_else(|| "https://example.com/test.jpg".to_string());

    let _ = test_operation(
        ext,
        src,
        "image_data",
        ErrorType::ImageData,
        config,
        &mut operations,
        |ext, src| {
            let data = ext.image_data(src, &test_image_url)?;
            Ok(data)
        },
    );

    // Test manga_update_details (getMangaUpdate with fetch_details=true)
    let _ = test_operation(
        ext,
        src,
        "manga_update_details",
        ErrorType::MangaUpdateDetails,
        config,
        &mut operations,
        |ext, src| {
            let details = ext.manga_update_details(src, &manga)?;
            Ok(details)
        },
    );

    // Test manga_update_chapters (getMangaUpdate with fetch_chapters=true)
    let _ = test_operation(
        ext,
        src,
        "manga_update_chapters",
        ErrorType::MangaUpdateChapters,
        config,
        &mut operations,
        |ext, src| {
            let chapters = ext.manga_update_chapters(src, &manga)?;
            Ok(chapters)
        },
    );

    // Test pages_coro (if we have chapters)
    if let Ok(chapters_vec) = &chapters_result {
        if let Some(chapter) = chapters_vec.first() {
            let _ = test_operation(
                ext,
                src,
                "pages_coro",
                ErrorType::PagesCoro,
                config,
                &mut operations,
                |ext, src| {
                    let pages = ext.pages_coro(src, chapter)?;
                    Ok(pages)
                },
            );
        }
    }

    let filter_results: Vec<FilterResult> = filters_for_search
        .into_iter()
        .map(|f| FilterResult {
            name: f.name,
            kind: format!("{:?}", f.kind),
            state: f.state,
            options: f.options,
        })
        .collect();

    Ok(SourceResult {
        name,
        lang,
        base_url,
        source_id,
        supports_latest,
        operations,
        filters: filter_results,
        error: source_error,
    })
}

fn test_operation<T, F>(
    ext: &mut Keiyoushi,
    src: &dexvm::keiyoushi::Source,
    op_name: &str,
    _error_type: ErrorType,
    config: &TestConfig,
    operations: &mut BTreeMap<String, OperationResult>,
    f: F,
) -> Result<T, JvmError>
where
    F: FnOnce(&mut Keiyoushi, &dexvm::keiyoushi::Source) -> Result<T, JvmError>,
    T: Debug,
{
    let _timeout = Duration::from_secs(config.timeout_secs);
    let start = Instant::now();
    let result = f(ext, src);
    let duration = start.elapsed().as_millis() as u64;

    // Warn if operation exceeded timeout (cannot actually interrupt VM)
    if duration > config.timeout_secs * 1000 {
        eprintln!(
            "  ⚠ {} exceeded timeout ({}ms > {}s)",
            op_name, duration, config.timeout_secs
        );
    }

    match result {
        Ok(ref data) => {
            let summary = summarize_result(op_name, data);
            let summary_clone = summary.clone();
            operations.insert(
                op_name.to_string(),
                OperationResult {
                    success: true,
                    duration_ms: duration,
                    data_summary: Some(summary),
                    error: None,
                    error_type: None,
                },
            );
            if config.verbose {
                println!("    ✓ {} ({}ms): {}", op_name, duration, summary_clone);
            }
        }
        Err(ref e) => {
            let err_str = ext.describe_error(e);
            let err_type = classify_error(e);
            operations.insert(
                op_name.to_string(),
                OperationResult {
                    success: false,
                    duration_ms: duration,
                    data_summary: None,
                    error: Some(err_str.clone()),
                    error_type: Some(err_type.as_str().to_string()),
                },
            );
            if config.verbose {
                println!(
                    "    ✗ {} ({}ms): {} [{}]",
                    op_name,
                    duration,
                    err_str,
                    err_type.as_str()
                );
            }
        }
    }

    result
}

fn mark_skipped(
    operations: &mut BTreeMap<String, OperationResult>,
    op_name: &str,
    _error_type: ErrorType,
    reason: &str,
) {
    operations.insert(
        op_name.to_string(),
        OperationResult {
            success: false,
            duration_ms: 0,
            data_summary: Some(format!("skipped: {}", reason)),
            error: Some(format!("skipped: {}", reason)),
            error_type: Some("skipped".to_string()),
        },
    );
}

fn get_test_manga(
    popular_result: &Result<MangaPages, JvmError>,
    search_result: &Result<MangaPages, JvmError>,
) -> Option<Manga> {
    popular_result
        .as_ref()
        .ok()
        .and_then(|p| p.mangas.first().cloned())
        .or_else(|| {
            search_result
                .as_ref()
                .ok()
                .and_then(|p| p.mangas.first().cloned())
        })
}

fn summarize_result(op_name: &str, data: &dyn std::fmt::Debug) -> String {
    // We can't easily match on the type here, so use a string representation
    let debug_str = format!("{:?}", data);
    // Extract key info based on operation
    match op_name {
        "filters" => {
            if let Some(n) = extract_count(&debug_str, "FilterDef") {
                format!("{} filters", n)
            } else {
                "filters".to_string()
            }
        }
        "popular" | "search" | "latest" | "popular_coro" | "search_coro" | "latest_coro" => {
            if let Some(n) = extract_count(&debug_str, "mangas") {
                format!(
                    "{} mangas, has_next={}",
                    n,
                    extract_bool(&debug_str, "has_next")
                )
            } else {
                op_name.to_string()
            }
        }
        "manga_details" | "manga_update_details" => {
            if let Some(title) = extract_field(&debug_str, "title") {
                format!("title: {}", title)
            } else {
                "details".to_string()
            }
        }
        "chapters" | "manga_update_chapters" => {
            if let Some(n) = extract_count(&debug_str, "Chapter") {
                format!("{} chapters", n)
            } else {
                "chapters".to_string()
            }
        }
        "pages" | "pages_coro" => {
            if let Some(n) = extract_count(&debug_str, "PageRef") {
                format!("{} pages", n)
            } else {
                "pages".to_string()
            }
        }
        "image_data" => {
            if let Some(n) = extract_bytes(&debug_str) {
                format!("{} bytes", n)
            } else {
                "image_data".to_string()
            }
        }
        _ => op_name.to_string(),
    }
}

fn extract_count(s: &str, key: &str) -> Option<usize> {
    // Try to find patterns like "mangas: [Manga {...}, Manga {...}]" -> count
    if let Some(start) = s.find(key) {
        let after = &s[start..];
        // Count occurrences of the struct pattern
        let count = after
            .matches("Manga {")
            .count()
            .max(after.matches("Chapter {").count())
            .max(after.matches("PageRef {").count());
        if count > 0 {
            return Some(count);
        }
    }
    None
}

fn extract_bool(s: &str, key: &str) -> String {
    if let Some(pos) = s.find(key) {
        let after = &s[pos + key.len()..];
        if after.starts_with(": true") || after.starts_with("=true") || after.starts_with("= true")
        {
            return "true".to_string();
        }
        if after.starts_with(": false")
            || after.starts_with("=false")
            || after.starts_with("= false")
        {
            return "false".to_string();
        }
    }
    "unknown".to_string()
}

fn extract_field(s: &str, field: &str) -> Option<String> {
    if let Some(pos) = s.find(field) {
        let after = &s[pos + field.len()..];
        if let Some(colon) = after.find(':') {
            let value = &after[colon + 1..];
            let end = value
                .find(',')
                .or_else(|| value.find('}'))
                .or_else(|| value.find(' '))
                .unwrap_or(value.len());
            let val = value[..end].trim().trim_matches('"');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn extract_bytes(s: &str) -> Option<usize> {
    // Look for Vec<u8> length
    if let Some(pos) = s.find("Vec<u8>") {
        let _after = &s[pos..];
        // Not easily extractable from Debug, skip
        return None;
    }
    None
}

fn classify_error(err: &JvmError) -> ErrorType {
    use dexvm::vm::error::JvmError;
    match err {
        JvmError::Resolution(_) => ErrorType::VmInit,
        JvmError::Decode(_) => ErrorType::VmInit,
        JvmError::Uncaught(_) => ErrorType::Unknown, // Could be anything (permission, network, etc.)
        JvmError::Fatal(_) => ErrorType::Unknown,
        _ => ErrorType::Unknown,
    }
}

/// Fallback for when we only have the error string (e.g., from describe_error)
fn classify_error_str(err: &str) -> ErrorType {
    let lower = err.to_lowercase();
    if lower.contains("permission") || lower.contains("denied") || lower.contains("sandbox") {
        ErrorType::Permission
    } else if lower.contains("http")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("timeout")
    {
        ErrorType::HttpExecution
    } else if lower.contains("jni") || lower.contains("unsupported") {
        ErrorType::VmInit
    } else if lower.contains("resolution")
        || lower.contains("not found")
        || lower.contains("missing")
    {
        ErrorType::VmInit
    } else if lower.contains("unimplemented") || lower.contains("stub") {
        ErrorType::VmInit
    } else {
        ErrorType::Unknown
    }
}

fn build_summary(extensions: &[ExtensionResult]) -> Summary {
    let mut sources_by_status = BTreeMap::new();
    let mut operations_by_status = BTreeMap::new();
    let mut failed_sources = Vec::new();
    let mut total_sources = 0;

    for ext in extensions {
        for src in &ext.sources {
            total_sources += 1;

            // Determine source status
            let has_success = src.operations.values().any(|op| op.success);
            let has_failure = src.error.is_some()
                || src
                    .operations
                    .values()
                    .any(|op| !op.success && op.error_type.as_deref() != Some("skipped"));
            let all_skipped = !has_success && !has_failure;

            let source_status = if has_success && !has_failure {
                "passed"
            } else if has_failure {
                "failed"
            } else if all_skipped {
                "skipped"
            } else {
                // Mixed: some passed, some skipped
                "passed"
            };

            *sources_by_status
                .entry(source_status.to_string())
                .or_default() += 1;

            if source_status == "failed" {
                let error_type = src.error.as_deref().unwrap_or("unknown").to_string();
                let error_message = src
                    .operations
                    .values()
                    .find(|op| !op.success && op.error_type.as_deref() != Some("skipped"))
                    .and_then(|op| op.error.as_deref())
                    .unwrap_or("unknown")
                    .to_string();
                failed_sources.push(FailedSourceSummary {
                    extension: ext.apk_name.clone(),
                    source: src.name.clone(),
                    error_type,
                    error_message,
                });
            }

            for (op_name, op) in &src.operations {
                let status = if op.success {
                    "passed"
                } else if op.error_type.as_deref() == Some("skipped") {
                    "skipped"
                } else {
                    "failed"
                };
                *operations_by_status
                    .entry(format!("{}:{}", op_name, status))
                    .or_default() += 1;
            }
        }
    }

    Summary {
        total_extensions: extensions.len(),
        total_sources,
        sources_by_status,
        operations_by_status,
        failed_sources,
    }
}

fn output_report(
    report: &TestReport,
    config: &TestConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Print human-readable summary
    println!("\n{}", "=".repeat(60));
    println!("TEST SUMMARY");
    println!("{}", "=".repeat(60));
    println!("Timestamp: {}", report.timestamp);
    if let Some(commit) = &report.git_commit {
        println!("Git commit: {}", commit);
    }
    if let Some(branch) = &report.git_branch {
        println!("Git branch: {}", branch);
    }
    println!("Extensions tested: {}", report.summary.total_extensions);
    println!("Total sources: {}", report.summary.total_sources);

    println!("\nSources by status:");
    for (status, count) in &report.summary.sources_by_status {
        println!("  {}: {}", status, count);
    }

    println!("\nOperations by status:");
    for (op_status, count) in &report.summary.operations_by_status {
        println!("  {}: {}", op_status, count);
    }

    if !report.summary.failed_sources.is_empty() {
        println!("\nFailed sources:");
        for fail in &report.summary.failed_sources {
            println!(
                "  [{}] {} / {} - {}: {}",
                fail.error_type, fail.extension, fail.source, fail.error_type, fail.error_message
            );
        }
    }

    // Per-extension detail
    println!("\n{}", "=".repeat(60));
    println!("PER-EXTENSION DETAIL");
    println!("{}", "=".repeat(60));
    for ext in &report.extensions {
        println!("\n{} ({})", ext.apk_name, ext.apk_path);
        if let Some(pkg) = &ext.manifest_package {
            println!("  Package: {}", pkg);
        }
        if let Some(name) = &ext.manifest_name {
            println!("  Name: {}", name);
        }
        if let Some(ver) = &ext.manifest_version {
            println!("  Version: {}", ver);
        }
        if let Some(err) = &ext.load_error {
            println!("  ✗ LOAD ERROR: {}", err);
            continue;
        }
        println!("  Sources: {}", ext.sources.len());
        for src in &ext.sources {
            let status = if src.error.is_some() { "✗" } else { "✓" };
            println!("    {} {} ({})", status, src.name, src.lang);
            for (op_name, op) in &src.operations {
                let op_status = if op.success {
                    "✓"
                } else if op.error_type.as_deref() == Some("skipped") {
                    "⊘"
                } else {
                    "✗"
                };
                let summary = op.data_summary.as_deref().unwrap_or("");
                let err = op.error.as_deref().unwrap_or("");
                println!(
                    "      {} {} ({}ms) {} {}",
                    op_status, op_name, op.duration_ms, summary, err
                );
            }
        }
        println!("  Duration: {}ms", ext.total_duration_ms);
    }

    // Write JSON if requested
    if let Some(json_path) = &config.json_output {
        let json = serde_json::to_string_pretty(report)?;
        fs::write(json_path, json)?;
        println!("\nJSON report written to: {}", json_path);
    }

    Ok(())
}

fn get_git_commit() -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn get_git_branch() -> Option<String> {
    std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn compare_reports(
    compare_path: &str,
    current_report: &TestReport,
    json_output: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load comparison report
    let compare_data = fs::read_to_string(compare_path)?;
    let compare_report: TestReport = serde_json::from_str(&compare_data)?;

    println!("Comparing with report: {}", compare_path);
    println!("Reference commit: {:?}", compare_report.git_commit);
    println!("Reference timestamp: {}", compare_report.timestamp);
    println!("Current commit: {:?}", current_report.git_commit);
    println!("Current timestamp: {}", current_report.timestamp);

    let mut regressions = Vec::new();
    let mut improvements = Vec::new();

    // Compare sources by status
    for ext in &current_report.extensions {
        let ref_ext = compare_report
            .extensions
            .iter()
            .find(|e| e.apk_name == ext.apk_name);
        for src in &ext.sources {
            let ref_src = ref_ext.and_then(|e| e.sources.iter().find(|s| s.name == src.name));

            let curr_status = source_status(src);
            let ref_status = ref_src.map(source_status).unwrap_or("missing".to_string());

            if curr_status == "failed" && ref_status == "passed" {
                regressions.push(format!(
                    "REGRESSION: {} / {} was {} now {}",
                    ext.apk_name, src.name, ref_status, curr_status
                ));
            } else if curr_status == "passed" && ref_status == "failed" {
                improvements.push(format!(
                    "IMPROVED: {} / {} was {} now {}",
                    ext.apk_name, src.name, ref_status, curr_status
                ));
            } else if curr_status != ref_status {
                regressions.push(format!(
                    "CHANGED: {} / {} was {} now {}",
                    ext.apk_name, src.name, ref_status, curr_status
                ));
            }

            // Compare operations
            if let Some(ref_src) = ref_src {
                for (op_name, curr_op) in &src.operations {
                    if let Some(ref_op) = ref_src.operations.get(op_name) {
                        let curr_ok = curr_op.success;
                        let ref_ok = ref_op.success;
                        if !curr_ok && ref_ok {
                            regressions.push(format!(
                                "REGRESSION: {} / {} / {} was pass now fail: {}",
                                ext.apk_name,
                                src.name,
                                op_name,
                                curr_op.error.as_deref().unwrap_or("")
                            ));
                        } else if curr_ok && !ref_ok {
                            improvements.push(format!(
                                "IMPROVED: {} / {} / {} was fail now pass",
                                ext.apk_name, src.name, op_name
                            ));
                        }
                    }
                }
            }
        }
    }

    // Print results
    println!("\n=== COMPARISON RESULTS ===");
    if regressions.is_empty() && improvements.is_empty() {
        println!("No changes detected.");
    } else {
        if !regressions.is_empty() {
            println!("\nREGRESSIONS ({}):", regressions.len());
            for r in &regressions {
                println!("  {}", r);
            }
        }
        if !improvements.is_empty() {
            println!("\nIMPROVEMENTS ({}):", improvements.len());
            for i in &improvements {
                println!("  {}", i);
            }
        }
    }

    // Summary counts
    let curr_passed = current_report
        .summary
        .sources_by_status
        .get("passed")
        .copied()
        .unwrap_or(0);
    let curr_failed = current_report
        .summary
        .sources_by_status
        .get("failed")
        .copied()
        .unwrap_or(0);
    let ref_passed = compare_report
        .summary
        .sources_by_status
        .get("passed")
        .copied()
        .unwrap_or(0);
    let ref_failed = compare_report
        .summary
        .sources_by_status
        .get("failed")
        .copied()
        .unwrap_or(0);

    println!("\nSource summary:");
    println!("  Reference: {} passed, {} failed", ref_passed, ref_failed);
    println!(
        "  Current:   {} passed, {} failed",
        curr_passed, curr_failed
    );
    println!(
        "  Delta:     {} passed, {} failed",
        curr_passed as i32 - ref_passed as i32,
        curr_failed as i32 - ref_failed as i32
    );

    if let Some(output) = json_output {
        let diff = serde_json::json!({
            "regressions": regressions,
            "improvements": improvements,
            "reference": compare_report.summary,
            "current": current_report.summary,
        });
        let json = serde_json::to_string_pretty(&diff)?;
        fs::write(output, json)?;
        println!("\nComparison written to: {}", output);
    }

    if !regressions.is_empty() {
        eprintln!("\n✗ {} regression(s) detected", regressions.len());
        std::process::exit(1);
    } else {
        println!("\n✓ No regressions");
    }

    Ok(())
}

fn source_status(src: &SourceResult) -> String {
    let has_success = src.operations.values().any(|op| op.success);
    let has_failure = src.error.is_some()
        || src
            .operations
            .values()
            .any(|op| !op.success && op.error_type.as_deref() != Some("skipped"));
    if has_success && !has_failure {
        "passed"
    } else if has_failure {
        "failed"
    } else {
        "skipped"
    }
    .to_string()
}
