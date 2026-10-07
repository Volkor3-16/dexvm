// Test if Jsoup can parse the HTML
use dom_query::Document;

fn main() {
    let html = std::fs::read_to_string("/home/volkor/git/dexvm/fixtures/live/tachiyomi-en.weebcentral-v1.6.25/002-weebcentral.com-search-data-text--sort-Popularity-order-Descending-official-Any-anime-Any-adult-").unwrap();
    let doc = Document::from(html.as_str());

    // Try to find manga articles
    let articles = doc.select("article.bg-base-300");
    println!("Found {} articles", articles.length());

    for article in articles.iter() {
        let title_elem = article.select("a.line-clamp-1");
        if title_elem.length() > 0 {
            println!("Title: {}", title_elem.first().text());
        }

        let href = article.select("a[href]");
        if href.length() > 0 {
            let url = href.first().attr("href").unwrap_or_default();
            println!("  URL: {}", url);
        }
    }
}
