use html5ever::parse_document;
use html5ever::tendril::TendrilSink;

use std::io;
use std::path::Path;
use std::default::Default;
use std::collections::HashMap;

use html5ever::driver::ParseOpts;
use html5ever::rcdom::RcDom;

use opengraph::Opengraph;
use schema_org::SchemaOrg;
use parser::Parser;

#[derive(Debug)]
pub struct HTML {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub feed: Option<String>,

    pub language: Option<String>, // as specified, not detected
    pub text_content: String, // all tags stripped from body

    pub meta: HashMap<String, String>, // flattened down list of meta properties
    pub opengraph: Opengraph,
    pub schema_org: Vec<SchemaOrg>,
}


impl HTML {
    fn empty(url: Option<String>) -> Self {
        Self {
            title: None,
            description: None,
            url,
            feed: None,

            language: None,
            text_content: String::new(),

            meta: HashMap::new(),
            opengraph: Opengraph::empty(),
            schema_org: Vec::new(),
        }
    }

    pub fn from_dom(dom: RcDom, url: Option<String>) -> Self {
        let mut html = Self::empty(url);
        let parser = Parser::start(dom.document);
        parser.traverse(&mut html);

        html
    }

    pub fn from_file(path: &str, url: Option<String>) -> Result<Self, io::Error> {
        let opts = ParseOpts {
            ..Default::default()
        };
        parse_document(RcDom::default(), opts)
            .from_utf8()
            .from_file(Path::new(path))
            .and_then(|dom| {
                Ok(Self::from_dom(dom, url))
            })
    }

    pub fn from_string(html: String, url: Option<String>) -> Result<Self, io::Error> {
        let opts = ParseOpts {
            ..Default::default()
        };
        parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .and_then(|dom| {
                Ok(Self::from_dom(dom, url))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::HTML;

    #[test]
    fn from_string() {
        let input = "<html><head><title>Hello</title></head><body>Contents".to_string();
        let html = HTML::from_string(input, None);
        assert!(html.is_ok());

        let html = html.unwrap();
        assert_eq!(html.title, Some("Hello".to_string()));
        assert!(html.description.is_none());
        assert_eq!(html.text_content, "Contents".to_string());
    }
}
