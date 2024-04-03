extern crate scraper;

use scraper::{ElementRef, Html, Selector};

pub fn check_url(html: &str, url: &str) -> bool {
    let document = Html::parse_document(html);
    // exclude status-not-ok
    let selector = Selector::parse("tr.status-ok").unwrap();
    let trs = document.select(&selector).collect::<Vec<ElementRef>>();

    let result = trs
        .into_iter()
        .filter(|tr| tr.text().collect::<Vec<_>>().join(" ").contains(url)) // Convert Text to string before using contains method
        .collect::<Vec<ElementRef>>();

    return result.len() > 0;
}
