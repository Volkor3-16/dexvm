# Download all keiyoushi extensions from GitHub releases and test them with exttest.

import os
import re
import sys
import subprocess
import urllib.request
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
import time
import threading

# Read URLs from browser output
with open('/tmp/pi-browser-81vjkJ/output.txt', 'r') as f:
    output = f.read()

urls = re.findall(r'https://github.com/keiyoushi/extensions/releases/download/[^"\s)]+\.apk', output)
unique_urls = sorted(set(urls))

print(f"Total unique extensions: {len(unique_urls)}")

# Download directory
DOWNLOAD_DIR = Path("/home/volkor/git/dexvm/fixtures/keiyoushi_all")
DOWNLOAD_DIR.mkdir(parents=True, exist_ok=True)

# Track downloads
downloaded = 0
failed = 0
skipped = 0
lock = threading.Lock()

def download_apk(url):
    global downloaded, failed, skipped
    filename = url.split('/')[-1]
    filepath = DOWNLOAD_DIR / filename
    
    if filepath.exists():
        with lock:
            skipped += 1
        return (url, True, "skipped (exists)")
    
    try:
        # Stream download using urllib
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req, timeout=120) as r:
            total_size = int(r.headers.get('content-length', 0))
            
            with open(filepath, 'wb') as f:
                while True:
                    chunk = r.read(8192)
                    if not chunk:
                        break
                    f.write(chunk)
        
        with lock:
            downloaded += 1
        return (url, True, f"downloaded ({total_size/1024/1024:.1f}MB)")
    except Exception as e:
        with lock:
            failed += 1
        if filepath.exists():
            filepath.unlink()
        return (url, False, str(e))

# Download with concurrency
print("Starting downloads...")
max_workers = 8  # Adjust based on bandwidth
with ThreadPoolExecutor(max_workers=max_workers) as executor:
    futures = {executor.submit(download_apk, url): url for url in unique_urls}
    
    for i, future in enumerate(as_completed(futures), 1):
        url, success, msg = future.result()
        if i % 50 == 0 or not success:
            print(f"[{i}/{len(unique_urls)}] {url.split('/')[-1]}: {msg}")

print(f"\nDownload complete:")
print(f"  Downloaded: {downloaded}")
print(f"  Skipped: {skipped}")
print(f"  Failed: {failed}")

# Run exttest on all downloaded APKs
if downloaded + skipped > 0:
    print("\nRunning exttest on all extensions...")
    apk_files = list(DOWNLOAD_DIR.glob("*.apk"))
    print(f"Testing {len(apk_files)} APKs...")
    
    # Test one by one and aggregate
    for apk in apk_files:
        print(f"Testing {apk.name}...")
        result = subprocess.run([
            "cargo", "run", "--features", "keiyoushi", "--bin", "exttest", "--",
            "--apk", str(apk),
            "--http", "empty",
            "--json", f"report_{apk.stem}.json"
        ], capture_output=True, text=True, timeout=300)
        
        # Check if test ran (exit code 0 = all passed, 1 = some failed, >1 = error)
        if result.returncode == 0:
            print(f"  OK (all sources passed)")
        elif result.returncode == 1:
            # Some sources failed but test completed - this is expected for many extensions
            # Extract failure count from stderr
            print(f"  PARTIAL (some sources failed)")
        else:
            print(f"  ERROR: {result.stderr[:200]}")

print("Done!")