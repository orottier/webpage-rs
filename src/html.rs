use html5ever::parse_document;
use html5ever::tendril::TendrilSink;

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

    pub language: Option<String>, // as specified, not detected
    pub text_content: String, // all tags stripped from body

    pub meta: HashMap<String, String>, // flattened down list of meta properties
    pub opengraph: Opengraph,
    pub schema_org: Vec<SchemaOrg>,
}


impl HTML {
    pub fn empty(url: Option<String>) -> Self {
        Self {
            title: None,
            description: None,
            url,

            language: None,
            text_content: String::new(),

            meta: HashMap::new(),
            opengraph: Opengraph::empty(),
            schema_org: Vec::new(),
        }
    }

    fn from_dom(dom: RcDom, url: Option<String>) -> Self {
        let mut html = Self::empty(url);
        let parser = Parser::start(dom.document);
        parser.traverse(&mut html);

        html
    }

    pub fn from_file(path: &str) -> Option<Self> {
        let opts = ParseOpts {
            ..Default::default()
        };
        parse_document(RcDom::default(), opts)
            .from_utf8()
            .from_file(Path::new(path))
            .and_then(|dom| {
                Ok(Self::from_dom(dom, None))
            }).ok()
    }

    pub fn from_string(html: String, url: Option<String>) -> Option<Self> {
        let opts = ParseOpts {
            ..Default::default()
        };
        parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .and_then(|dom| {
                Ok(Self::from_dom(dom, url))
            }).ok()
    }
}
