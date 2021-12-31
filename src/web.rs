use reqwest::blocking;
use tracing::{span, Level};
use url::Url;

use crate::css;
use crate::css::Stylesheet;
use crate::html::{self, DOMContent, DOMElement};
use crate::style::StyledElement;

#[derive(Debug)]
pub struct Page {
    url: Url,
    dom: DOMElement,
    pub style_tree: StyledElement,
}

impl Page {
    pub fn browse(url: impl Into<String>) -> Self {
        let url = Url::parse(url.into().as_str()).expect("Could not parse URL");
        let resp = Page::get_text_resource(url.clone()).expect("Could not get page");
        let doc = html::document(resp.as_str())
            .expect("Could not parse HTML")
            .1;
        let mut page = Self::from_dom(doc, url);
        let styles = page.get_styles();
        styles
            .into_iter()
            .for_each(|sheet| page.style_tree.apply_styles(sheet.rules));
        page
    }

    pub fn from_dom(dom: DOMElement, url: Url) -> Self {
        let style = dom.clone().into();
        Self {
            url,
            dom,
            style_tree: style,
        }
    }

    pub fn get_styles(&self) -> Vec<Stylesheet> {
        let mut sheets = Vec::new();
        if let Some(head) = self.dom.get_elements_by_name("head", false).get(0) {
            head.get_elements_by_name("link", false)
                .iter()
                .filter(|l| l.get_attribute("rel") == Some(&"stylesheet".into()))
                .for_each(|s| {
                    if let Some(href) = s.get_attribute("href") {
                        if let Ok(resource) = self.get_linked_text_resource(href) {
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

    fn get_text_resource(url: impl Into<String>) -> Result<String, reqwest::Error> {
        let url = Url::parse(url.into().as_str()).expect("Could not parse URL");
        let span = span!(Level::DEBUG, "Loading resource", "{}", &url);
        let _enter = span.enter();
        if url.scheme() == "file" {
            Ok(std::fs::read_to_string(url.path()).expect("Could not access file"))
        } else {
            blocking::get(url).expect("Could not request URL").text()
        }
    }

    fn get_linked_text_resource(&self, url: impl Into<String>) -> Result<String, reqwest::Error> {
        let url = self.resolve_url(url).expect("Could not resolve URL");
        Page::get_text_resource(url)
    }
}
