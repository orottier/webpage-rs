use std::collections::HashMap;

#[derive(Debug)]
pub struct Object {
    pub url: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Opengraph {
    pub og_type: String,
    pub properties: HashMap<String, String>,

    pub images: Vec<Object>,
    pub videos: Vec<Object>,
    pub audios: Vec<Object>,
}

impl Object {
    pub fn new(url: String) -> Self {
        Self {
            url,
            properties: HashMap::new(),
        }
    }
}

impl Opengraph {
    pub fn empty() -> Self {
        Self {
            og_type: "website".to_string(),
            properties: HashMap::new(),

            images: vec![],
            videos: vec![],
            audios: vec![],
        }
    }

    pub fn extend(&mut self, property: &str, content: String) -> () {
        if property == "type" {
            self.og_type = content;
        } else if property.starts_with("image") {
            parse_object("image", property, content, &mut self.images);
        } else if property.starts_with("video") {
            parse_object("video", property, content, &mut self.videos);
        } else if property.starts_with("audio") {
            parse_object("audio", property, content, &mut self.audios);
        } else {
            self.properties.insert(property.to_string(), content);
        }
    }
}

fn parse_object(og_type: &str, property: &str, content: String, collection: &mut Vec<Object>) -> () {
    let num_images = collection.len();

    if property == og_type || &property[og_type.len()..] == ":url" {
        collection.push(Object::new(content));
    } else if num_images > 0 && property.len() > og_type.len() + 1 {
        let property = &property["image:".len()..];
        collection[num_images - 1]
            .properties.insert(property.to_string(), content);
    }
}

