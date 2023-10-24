//! OpenGraph information

use std::collections::HashMap;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
/// Representing [OpenGraph](http://ogp.me/) information
pub struct Opengraph {
    /// Opengraph type (article, image, event, ..)
    pub og_type: String,
    /// Opengraph properties of this object
    pub properties: HashMap<String, String>,

    /// Images relevant to this object
    pub images: Vec<OpengraphObject>,
    /// Videos relevant to this object
    pub videos: Vec<OpengraphObject>,
    /// Audio relevant to this object
    pub audios: Vec<OpengraphObject>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
/// Info about an OpenGraph media type
pub struct OpengraphObject {
    /// URL describing this object
    pub url: String,
    /// Properties of the referred object
    pub properties: HashMap<String, String>,
}

impl OpengraphObject {
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

    pub fn extend(&mut self, property: &str, content: String) {
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

fn parse_object(
    og_type: &str,
    property: &str,
    content: String,
    collection: &mut Vec<OpengraphObject>,
) {
    let num_images = collection.len();

    if property == og_type || &property[og_type.len()..] == ":url" {
        collection.push(OpengraphObject::new(content));
    } else if num_images > 0 && property.len() > og_type.len() + 1 {
        let property = &property["image:".len()..];
        collection[num_images - 1]
            .properties
            .insert(property.to_string(), content);
    }
}

#[cfg(test)]
mod tests {
    use super::Opengraph;

    #[test]
    fn test_type() {
        let mut opengraph = Opengraph::empty();
        assert_eq!(opengraph.og_type, "website");

        opengraph.extend("type", "article".to_string());
        assert_eq!(opengraph.og_type, "article");
    }

    #[test]
    fn test_image() {
        let mut opengraph = Opengraph::empty();

        opengraph.extend("image", "http://example.org/image.png".to_string());
        opengraph.extend(
            "image:secure_url",
            "https://example.org/image.png".to_string(),
        );
        assert_eq!(opengraph.images.len(), 1);
        assert_eq!(opengraph.images[0].url, "http://example.org/image.png");

        let prop = opengraph.images[0].properties.get("secure_url");
        assert!(prop.is_some());
        assert_eq!(prop.unwrap(), "https://example.org/image.png");
    }
}
