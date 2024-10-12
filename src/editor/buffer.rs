pub struct Buffer {
    lines: Vec<String>,
}

pub fn buffer_lines(buffer: &Buffer) -> &Vec<String> {
    &buffer.lines
}

impl Buffer {
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}
