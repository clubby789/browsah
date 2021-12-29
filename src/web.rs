use reqwest::blocking;
use url::Url;

use crate::html::{self, DOMElement, DOMContent};
use crate::style::StyledElement;
use crate::css::Stylesheet;
use crate::css;

#[derive(Debug)]
pub struct Page {
    url: Url,
    dom: DOMElement,
    style: StyledElement
}

impl Page {
    pub fn browse(url: impl Into<String>) -> Self {
        let url = Url::parse(url.into().as_str()).expect("Could not parse URL");
        let resp = blocking::get(url.clone())
            .expect("Could not request URL")
            .text()
            .expect("Could not get response text");
        let doc = html::document(resp.as_str())
            .expect("Could not parse HTML")
            .1;
        let mut page = Self::from_dom(doc, url);
        let styles = page.get_styles();
        styles.into_iter().for_each(|sheet| page.style.apply_styles(sheet.rules));
        page
    }

    pub fn from_dom(dom: DOMElement, url: Url) -> Self {
        let style = dom.clone().into();
        Self {url, dom, style}
    }

    pub fn get_styles(&self) -> Vec<Stylesheet> {
        let mut sheets = Vec::new();
        if let Some(head) = self.dom.get_elements_by_name("head", false).get(0) {
            head.get_elements_by_name("link", false)
                .iter()
                .filter(|l| l.get_attribute("rel") == Some(&"stylesheet".into()))
                .for_each(|s| {
                    if let Some(href) = s.get_attribute("href") {
                        if let Ok(resource) = self.get_text_resource(href) {
                            let sheet = css::stylesheet(resource.as_str())
                                .expect("Could not parse CSS")
                                .1;
                            sheets.push(sheet);
                        }
                    }
                });
            head.get_elements_by_name("style", false)
                .iter()
                .for_each(|e| {
                    if let Some(DOMContent::Text(t)) = e.contents.get(0) {
                        if let Ok((_, sheet)) = css::stylesheet(t.as_str()) {
                            sheets.push(sheet);
                        }
                    }
                })
        }
        sheets
    }

    fn resolve_url(&self, url: impl Into<String>) -> Result<Url, url::ParseError> {
        let url: String = url.into();
        self.url.join(url.as_str())
    }

    fn get_text_resource(&self, url: impl Into<String>) -> Result<String, reqwest::Error> {
        let url = self.resolve_url(url.into()).unwrap();
        let response = blocking::get(url)?;
        response.text()
    }
}