use html5ever::{parse_document, Attribute};

use std::path::Path;
use std::default::Default;

use html5ever::driver::ParseOpts;
use html5ever::rcdom::{NodeData, RcDom, Handle};
use html5ever::tendril::{Tendril, fmt::UTF8, TendrilSink};

use opengraph::Opengraph;

#[derive(Debug)]
pub struct HTML {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub text_content: String,

    pub opengraph: Opengraph,
}

struct ParserState<'a> {
    in_body: bool,
    parent: Option<&'a NodeData>,
}

impl HTML {
    pub fn empty(url: Option<String>) -> Self {
        Self {
            title: None,
            description: None,
            url,
            text_content: String::new(),

            opengraph: Opengraph::empty(),
        }
    }

    fn from_dom(dom: RcDom, url: Option<String>) -> Self {
        let mut html = Self::empty(url);
        let state = ParserState {
            in_body: false,
            parent: None,
        };
        traverse(dom.document, state, &mut html);

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

fn traverse(handle: Handle, state: ParserState, html: &mut HTML) -> () {
    let mut in_body = state.in_body;

    match handle.data {
        NodeData::Document => (),
        NodeData::Doctype { .. } => (),
        NodeData::Text { ref contents } => {
            if in_body {
                if let Some(NodeData::Element { ref name, .. }) = state.parent {
                    if name.local.as_ref() != "style" && name.local.as_ref() != "script" {
                        html.text_content.push_str(tendril_to_utf8(&contents.borrow()));
                    }
                }
            }
            return;
        },
        NodeData::Comment { .. } => (),

        NodeData::Element { ref name, ref attrs, .. } => {
            if name.local.as_ref() == "body" {
                in_body = true;
            }

            if name.local.as_ref() == "title" {
                html.title = text_content(&handle);
            }
            if name.local.as_ref() == "meta" {
                let property = get_attribute(&attrs.borrow(), "property")
                    .unwrap_or(get_attribute(&attrs.borrow(), "name")
                        .unwrap_or("".to_string())
                    );

                if property == "description" {
                    let content = get_attribute(&attrs.borrow(), "content");
                    html.description = content;
                } else if property.starts_with("og:") && property.len() > 3 {
                    let content = get_attribute(&attrs.borrow(), "content");
                    if let Some(content) = content {
                        html.opengraph.extend(&property[3..], content);
                    }
                }
            }
            if name.local.as_ref() == "link" {
                if get_attribute(&attrs.borrow(), "rel").unwrap_or("".to_string()) == "canonical" {
                    html.url = get_attribute(&attrs.borrow(), "href");
                }
            }
        }

        NodeData::ProcessingInstruction { .. } => unreachable!()
    }

    for child in handle.children.borrow().iter() {
        let new_state = ParserState {
            in_body,
            parent: Some(&handle.data),
        };
        traverse(child.clone(), new_state, html);
    }
}

fn get_attribute(attrs: &Vec<Attribute>, name: &str) -> Option<String> {
    attrs.iter()
        .filter(|attr| attr.name.local.as_ref() == name)
        .nth(0)
        .and_then(|attr| Some(attr.value.trim().to_string()))
}

fn text_content(handle: &Handle) -> Option<String> {
    // todo paste all the text together
    for child in handle.children.borrow().iter() {
        if let NodeData::Text { ref contents } = child.data {
            let string = tendril_to_utf8(&contents.borrow()).to_string();
            return Some(string.trim().to_string());
        }
    }

    None
}

fn tendril_to_utf8(t: &Tendril<UTF8>) -> &str {
    t
}
