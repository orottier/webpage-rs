
use html5ever::{parse_document, Attribute};

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

    pub opengraph: Opengraph,
}

impl HTML {
    pub fn empty(url: &str) -> Self {
        Self {
            title: None,
            description: None,
            url: Some(url.to_string()),

            opengraph: Opengraph::empty(),
        }
    }

    pub fn from_string(html: String, url: &str) -> Option<Self> {
        let opts = ParseOpts {
            ..Default::default()
        };
        parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .and_then(|dom| {
                let mut html = Self::empty(url);
                traverse(dom.document, &mut html);

                Ok(html)
            }).ok()
    }
}

fn traverse(handle: Handle, html: &mut HTML) -> () {
    match handle.data {
        NodeData::Document => (),
        NodeData::Doctype { .. } => (),
        NodeData::Text { .. } => (),
        NodeData::Comment { .. } => (),

        NodeData::Element { ref name, ref attrs, .. } => {
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
        traverse(child.clone(), html);
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
