use std::fmt;

/// A Clipping is the representaion of data taken from the clipboard
///
/// For now clippings only store utf-8 text
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Clipping {
    pub data: String,
    pub id: usize,
}

impl From<String> for Clipping {
    fn from(string: String) -> Self {
        Clipping {
            data: string,
            id: 0,
        }
    }
}

impl fmt::Display for Clipping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{id: {} | '{}'}}", &self.id, &self.data)
    }
}
