use std::cell::RefCell;
use tracing::{span, Level};
use url::Url;

use crate::style::{StyledElement, USER_AGENT_CSS};
use css::Stylesheet;
use html::{self, DOMContent, DOMElement};

pub struct Page<'a> {
    url: Url,
    dom: DOMElement,
    pub style_tree: RefCell<StyledElement<'a>>,
}

impl<'a> Page<'a> {
    /// Browses to and parses a web page without applying style information (except for the default)
    pub fn browse(url: &str) -> Self {
        let url = Url::parse(url).expect("Could not parse URL");
        let resp = Page::get_text_resource(url.as_str()).expect("Could not get page");
        let doc = html::document(resp.as_str())
            .expect("Could not parse HTML")
            .1;
        let page = Self::from_dom(doc, url);
        page.style_tree
            .borrow_mut()
            .apply_styles(&USER_AGENT_CSS.rules);
        page
    }

    pub fn from_dom(dom: DOMElement, url: Url) -> Self {
        let style = RefCell::new(dom.clone().into());
        Self {
            url,
            dom,
            style_tree: style,
        }
    }

    pub fn get_stylesheet_text(&self) -> Vec<String> {
        let mut sheets = Vec::new();
        if let Some(head) = self.dom.get_elements_by_name("head", false).get(0) {
            head.get_elements_by_name("link", false)
                .iter()
                .filter(|l| l.get_attribute("rel") == Some(&"stylesheet".into()))
                .for_each(|s| {
                    if let Some(href) = s.get_attribute("href") {
                        if let Ok(resource) = self.get_linked_text_resource(href) {
                            sheets.push(resource);
                        }
                    }
                });
            head.get_elements_by_name("style", false)
                .iter()
                .for_each(|e| {
                    if let Some(DOMContent::Text(t)) = e.contents.get(0).cloned() {
                        sheets.push(t);
                    }
                })
        }
        sheets
    }

    pub fn get_styles(&'a self, styles: &'a [String]) -> Vec<Stylesheet> {
        styles
            .iter()
            .filter_map(|s| css::stylesheet(s).ok())
            .map(|(_, s)| s)
            .collect()
    }

    fn resolve_url(&self, url: &str) -> Result<Url, url::ParseError> {
        self.url.join(url)
    }

    fn get_text_resource(url: &str) -> Result<String, ureq::Error> {
        let url = Url::parse(url).expect("Could not parse URL");
        let span = span!(Level::DEBUG, "Loading resource", "{}", &url);
        let _enter = span.enter();
        if url.scheme() == "file" {
            Ok(std::fs::read_to_string(url.path()).expect("Could not access file"))
        } else {
            Ok(ureq::get(url.as_str())
                .call()?
                .into_string()
                .expect("Could not get text"))
        }
    }

    fn get_linked_text_resource(&self, url: &str) -> Result<String, ureq::Error> {
        let url = self.resolve_url(url).expect("Could not resolve URL");
        Page::get_text_resource(url.as_str())
    }
}
