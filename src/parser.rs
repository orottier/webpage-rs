use html5ever::tendril::{fmt::UTF8, Tendril};
use html5ever::Attribute;
use markup5ever_rcdom::{Handle, NodeData};

use crate::html::{Link, HTML};
use crate::schema_org::SchemaOrg;

#[derive(Copy, Clone)]
enum Segment {
    None,
    Head,
    Body,
}

pub struct Parser<'a> {
    segment: Segment,
    parent: Option<&'a NodeData>,
    handle: Handle,
}

impl<'a> Parser<'a> {
    pub fn start(handle: Handle) -> Self {
        Parser {
            handle,
            segment: Segment::None,
            parent: None,
        }
    }

    pub fn traverse(self, html: &mut HTML) {
        let mut segment = self.segment;

        let handle_ref = &self.handle;
        match self.handle.data {
            NodeData::Document => (),
            NodeData::Doctype { .. } => (),
            NodeData::Comment { .. } => (),

            NodeData::Text { ref contents } => {
                if let Some(NodeData::Element { ref name, .. }) = self.parent {
                    let tag_name = name.local.as_ref();

                    process_text(
                        self.segment,
                        tag_name,
                        tendril_to_utf8(&contents.borrow()),
                        html,
                    )
                }
            }

            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let tag_name = name.local.as_ref();

                if tag_name == "head" {
                    segment = Segment::Head;
                } else if tag_name == "body" {
                    segment = Segment::Body;
                }

                process_element(segment, tag_name, handle_ref, &attrs.borrow(), html)
            }

            NodeData::ProcessingInstruction { .. } => unreachable!(),
        }

        for child in self.handle.children.borrow().iter() {
            let new_parser = Parser {
                segment,
                parent: Some(&self.handle.data),
                handle: child.clone(),
            };
            new_parser.traverse(html);
        }
    }
}

fn process_text(segment: Segment, tag_name: &str, contents: &str, html: &mut HTML) {
    if let Segment::Body = segment {
        if tag_name != "style" && tag_name != "script" && tag_name != "noscript" {
            if !html.text_content.is_empty() {
                html.text_content.push(' ');
            }
            html.text_content.push_str(contents);
        }
    }
}

fn process_element(
    segment: Segment,
    tag_name: &str,
    handle: &Handle,
    attrs: &[Attribute],
    html: &mut HTML,
) {
    // process language attribute
    if tag_name == "html" || tag_name == "body" {
        let language = get_attribute(attrs, "lang");
        if language.is_some() {
            html.language = language;
        }
    }

    // process <head>
    if let Segment::Head = segment {
        if tag_name == "title" {
            html.title = text_content(handle);
        }
        if tag_name == "meta" {
            let content = get_attribute(attrs, "content");
            if let Some(content) = content {
                let property_opt = get_attribute(attrs, "property")
                    .or_else(|| get_attribute(attrs, "name"))
                    .or_else(|| get_attribute(attrs, "http-equiv"));

                if let Some(property) = property_opt {
                    html.meta.insert(property.clone(), content.clone());

                    if property.starts_with("og:") && property.len() > 3 {
                        html.opengraph.extend(&property[3..], content);
                    } else if property == "description" {
                        html.description = Some(content);
                    }
                }
            }

            if let Some(charset) = get_attribute(attrs, "charset") {
                html.meta.insert("charset".to_string(), charset);
            }
        }
        if tag_name == "link" {
            let rel = get_attribute(attrs, "rel").unwrap_or_default();
            if rel == "canonical" {
                html.set_url(get_attribute(attrs, "href"));
            } else if rel == "alternate" {
                let link_type = get_attribute(attrs, "type").unwrap_or_default();
                if [
                    "application/atom+xml",
                    "application/json",
                    "application/rdf+xml",
                    "application/rss+xml",
                    "application/xml",
                    "text/xml",
                ]
                .contains(&&link_type[..])
                {
                    html.feed = get_attribute(attrs, "href");
                }
            }
        }
    }

    // process ld-json snippets
    if tag_name == "script" {
        if let Some(script_type) = get_attribute(attrs, "type") {
            if script_type == "application/ld+json" {
                if let Some(content) = text_content(handle) {
                    html.schema_org.append(&mut SchemaOrg::from(content));
                }
            }
        }
    }

    if tag_name == "a" {
        if let Some(href) = get_attribute(attrs, "href") {
            let text = text_content(handle).unwrap_or_default();
            let href = if let Some(url) = &html.url_parsed {
                if let Ok(url) = url.join(&href) {
                    url.to_string()
                } else {
                    href
                }
            } else {
                href
            };
            html.links.push(Link { url: href, text });
        }
    }
}

fn get_attribute(attrs: &[Attribute], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.name.local.as_ref() == name)
        .map(|attr| attr.value.trim().to_string())
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
