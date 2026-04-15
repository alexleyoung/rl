const CHUNK_SIZE: usize = 1000;

pub fn chunk_text(text: &str) -> Vec<String> {
    chunk_text_sized(text, CHUNK_SIZE)
}

pub fn chunk_text_sized(text: &str, size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut buf = String::with_capacity(size);
    for word in text.split_whitespace() {
        if buf.len() + word.len() + 1 > size && !buf.is_empty() {
            chunks.push(buf.trim().to_string());
            buf.clear();
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(word);
    }
    if !buf.trim().is_empty() {
        chunks.push(buf.trim().to_string());
    }
    chunks
}
