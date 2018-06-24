use html5ever::{parse_document, Attribute};

use std::path::Path;
use std::default::Default;
use std::collections::HashMap;

use html5ever::driver::ParseOpts;
use html5ever::rcdom::{NodeData, RcDom, Handle};
use html5ever::tendril::{Tendril, fmt::UTF8, TendrilSink};

use opengraph::Opengraph;

#[derive(Debug)]
pub struct HTML {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,

    pub language: Option<String>, // as specified, not detected
    pub text_content: String, // all tags stripped from body

    pub meta: HashMap<String, String>, // flattened down list of meta properties
    pub opengraph: Opengraph,
}

#[derive(Copy, Clone)]
enum Segment {
    None,
    Head,
    Body,
}
struct ParserState<'a> {
    segment: Segment,
    parent: Option<&'a NodeData>,
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
        }
    }

    fn from_dom(dom: RcDom, url: Option<String>) -> Self {
        let mut html = Self::empty(url);
        let state = ParserState {
            segment: Segment::None,
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
    let mut segment = state.segment;

    match handle.data {
        NodeData::Document => (),
        NodeData::Doctype { .. } => (),
        NodeData::Text { ref contents } => {
            if let Segment::Body = segment {
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
            let tag_name = name.local.as_ref();

            if tag_name == "head" {
                segment = Segment::Head;
            } else if tag_name == "body" {
                segment = Segment::Body;
            }

            if tag_name == "html" || tag_name == "body" {
                let language = get_attribute(&attrs.borrow(), "lang");
                if language.is_some() {
                    html.language = language;
                }
            }

            if let Segment::Head = segment {
                if tag_name == "title" {
                    html.title = text_content(&handle);
                }
                if tag_name == "meta" {
                    let content = get_attribute(&attrs.borrow(), "content");
                    if let Some(content) = content {
                        let property_opt = get_attribute(&attrs.borrow(), "property")
                            .or(get_attribute(&attrs.borrow(), "name"))
                            .or(get_attribute(&attrs.borrow(), "http-equiv"));

                        if let Some(property) = property_opt {
                            html.meta.insert(property.clone(), content.clone());

                            if property.starts_with("og:") && property.len() > 3 {
                                html.opengraph.extend(&property[3..], content);
                            } else if property == "description" {
                                html.description = Some(content);
                            }
                        }
                    }

                    if let Some(charset) = get_attribute(&attrs.borrow(), "charset") {
                        html.meta.insert("charset".to_string(), charset);
                    }
                }
                if tag_name == "link" {
                    if get_attribute(&attrs.borrow(), "rel").unwrap_or("".to_string()) == "canonical" {
                        html.url = get_attribute(&attrs.borrow(), "href");
                    }
                }
            }
        }

        NodeData::ProcessingInstruction { .. } => unreachable!()
    }

    for child in handle.children.borrow().iter() {
        let new_state = ParserState {
            segment,
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
