/// A Clipping is the representaion of data taken from the clipboard
///
/// For now clippings only store utf-8 text
#[derive(Debug, Serialize, Deserialize)]
pub struct Clipping(String);
