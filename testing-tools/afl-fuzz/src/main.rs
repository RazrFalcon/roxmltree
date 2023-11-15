fn main() {
    afl::fuzz!(|data: &[u8]| {
        if let Ok(text) = std::str::from_utf8(data) {
            let _ = roxmltree::Document::parse(&text);
        }
    });
}
