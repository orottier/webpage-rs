//! Schema.org information

use serde_json::{self, Value};

/// Representing [Schema.org](https://schema.org/) information (currently only via JSON-LD)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct SchemaOrg {
    /// Schema.org type (article, image, event)
    pub schema_type: String,
    /// Schema.org info
    pub value: Value,
}

impl SchemaOrg {
    pub fn from(content: String) -> Vec<Self> {
        let node: Value = serde_json::from_str(&content).unwrap_or(Value::Null);

        let vals: Vec<Value>;
        if let Value::Array(arr) = node {
            vals = arr;
        } else {
            vals = vec![node];
        }

        // Some websites place schema.org objects under "@graph", which we want to use as values
        let vals: Vec<Value> = if let Some(obj) = vals.first().and_then(|v| v.as_object()) {
            obj.get("@graph")
                .and_then(|v| v.as_array())
                .unwrap_or(&vals)
                .to_vec()
        } else {
            vals
        };

        vals.into_iter()
            .flat_map(|v| {
                let type_opt = v["@type"].clone();
                if let Value::String(ref type_val) = type_opt {
                    return Some(SchemaOrg {
                        schema_type: type_val.to_string(),
                        value: v,
                    });
                }
                None
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaOrg;

    #[test]
    fn test_empty() {
        let schema = SchemaOrg::from("{}".to_string());
        assert!(schema.is_empty());
    }

    #[test]
    fn test_type() {
        let schema = SchemaOrg::from("{\"@type\": \"NewsArticle\"}".to_string());
        assert_eq!(schema.len(), 1);
        assert_eq!(schema[0].schema_type, "NewsArticle");
    }

    #[test]
    fn test_graph() {
        let schema = SchemaOrg::from("{\"@context\":\"https://schema.org\",\"@graph\":[{\"@context\":\"https://schema.org\",\"@type\":\"NewsArticle\"}]}".to_string());
        assert_eq!(schema.len(), 1);
        assert_eq!(schema[0].schema_type, "NewsArticle");
    }
}
