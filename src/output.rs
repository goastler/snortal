use crate::types::{PortalUrl, StatusNote};
use std::collections::HashSet;

fn normalize(url: &str) -> String {
    match url::Url::parse(url) {
        Ok(u) => {
            let mut s = u.to_string();
            // Strip trailing slash for dedup comparison (http://x/ == http://x)
            if s.ends_with('/') && u.path() == "/" {
                s.pop();
            }
            s
        }
        Err(_) => url.to_owned(),
    }
}

pub fn deduplicate_and_sort(mut urls: Vec<PortalUrl>) -> Vec<PortalUrl> {
    urls.sort_by(|a, b| b.confidence.cmp(&a.confidence));
    let mut seen = HashSet::new();
    urls.retain(|p| seen.insert(normalize(&p.url)));
    urls
}

pub fn print_results(urls: Vec<PortalUrl>, notes: Vec<StatusNote>, json: bool, verbose: bool) {
    let urls = deduplicate_and_sort(urls);

    if json {
        print_json(&urls, &notes);
        return;
    }

    if urls.is_empty() {
        println!("No captive portal URLs detected.");
    } else {
        println!("Captive portal URLs detected:");
        println!("{}", "\u{2500}".repeat(60));
        for p in &urls {
            println!(" {:>3}  {:<45}  [{}]", p.confidence, p.url, p.source);
        }
        println!("{}", "\u{2500}".repeat(60));
    }

    for note in &notes {
        println!("Status: {}", note.message);
    }

    if verbose && urls.is_empty() && notes.is_empty() {
        println!("(no detectors found anything)");
    }
}

fn print_json(urls: &[PortalUrl], notes: &[StatusNote]) {
    print!("[");
    for (i, p) in urls.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        let source = p.source.to_string().replace('"', "\\\"");
        let url = p.url.replace('"', "\\\"");
        print!(
            "{{\"url\":\"{url}\",\"source\":\"{source}\",\"confidence\":{}}}",
            p.confidence
        );
    }
    for (i, n) in notes.iter().enumerate() {
        if !urls.is_empty() || i > 0 {
            print!(",");
        }
        let msg = n.message.replace('"', "\\\"");
        print!("{{\"status\":\"{msg}\"}}");
    }
    println!("]");
}
