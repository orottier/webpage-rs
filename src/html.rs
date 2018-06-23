
use html5ever::parse_document;

use std::io;
use std::default::Default;
use html5ever::driver::ParseOpts;
use html5ever::rcdom::{NodeData, RcDom, Handle};
use html5ever::tendril::{Tendril, fmt::UTF8, TendrilSink};

pub struct HTML {
    pub title: String,
}

impl HTML {
    pub fn from_string(html: String) -> Option<Self> {
        let opts = ParseOpts {
            ..Default::default()
        };
        parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .and_then(|dom| {
                if let Some(title) = find_title(dom.document) {
                    Ok(Self {
                        title,
                    })
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "No title found"))
                }
            }).ok()
    }
}

fn find_title(handle: Handle) -> Option<String> {
    match handle.data {
        NodeData::Document
            => (),

        NodeData::Doctype { .. }
            => return None,

        NodeData::Text { .. }
            => return None,

        NodeData::Comment { .. }
            => return None,

        NodeData::Element { ref name, ref attrs, .. } => {
            if name.local.as_ref() == "title" {
                return text_content(&handle);
            }
        }

        NodeData::ProcessingInstruction { .. } => unreachable!()
    }

    for child in handle.children.borrow().iter() {
        if let Some(title) = find_title(child.clone()) {
            return Some(title);
        }
    }

    None
}

fn text_content(handle: &Handle) -> Option<String> {
    // todo paste all the text together
    for child in handle.children.borrow().iter() {
        if let NodeData::Text { ref contents } = child.data {
            let string = tendril_to_utf8(&contents.borrow()).to_string();
            return Some(string.to_string());
        }
    }

    None
}

fn tendril_to_utf8(t: &Tendril<UTF8>) -> &str {
    t
}
