use parquet::record::{Row, RowAccessor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct Email {
    pub file: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug)]
pub struct OriginalEmail {
    pub file: String,
    pub message: String,
}

impl From<Row> for OriginalEmail {
    fn from(row: Row) -> Self {
        // I just happen to know the order
        let file = row.get_string(0).expect("No file field found").to_owned();
        let message = row
            .get_string(1)
            .expect("No message field found")
            .to_owned();

        OriginalEmail { file, message }
    }
}

impl From<OriginalEmail> for Email {
    fn from(original: OriginalEmail) -> Self {
        let mut in_headers = true;
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body = String::new();
        let mut prev_header: &str = "";

        for line in original.message.split('\n') {
            if in_headers {
                if line.is_empty() {
                    in_headers = false;
                    continue;
                }

                match line.split_once(':') {
                    Some((key, value)) => {
                        headers.insert(
                            key.to_string(),
                            value.strip_prefix(' ').unwrap_or_default().to_string(),
                        );
                        prev_header = key;
                    }
                    None => {
                        let line = line.strip_prefix(' ').unwrap_or_default();
                        headers.get_mut(prev_header).unwrap().push_str(line);
                    }
                }
            } else {
                if !body.is_empty() {
                    body.push('\n');
                }
                body.push_str(line);
            }
        }

        Email {
            file: Some(original.file),
            body,
            headers,
        }
    }
}
