use serde_json::{self, Value};

#[derive(Debug)]
pub struct SchemaOrg {
    pub schema_type: String,
    pub value: Value,
}

impl SchemaOrg {
    pub fn from(content: String) -> Option<Self> {
        let v: Value = serde_json::from_str(&content).unwrap_or(Value::Null);

        let type_opt = v["@type"].clone();
        if let Value::String(ref type_val) = type_opt {
            return Some(SchemaOrg {
                schema_type: type_val.to_string(),
                value: v,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaOrg;

    #[test]
    fn test_empty() {
        let schema = SchemaOrg::from("{}".to_string());
        assert!(schema.is_none());
    }

    #[test]
    fn test_type() {
        let schema = SchemaOrg::from("{\"@type\": \"Test\"}".to_string());
        assert!(schema.is_some());
        assert_eq!(schema.unwrap().schema_type, "Test");
    }
}
