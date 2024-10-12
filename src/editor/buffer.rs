use std::fs;

pub struct Buffer {
    lines: Vec<String>,
    file_path: Option<String>,
}

pub fn buffer_lines(buffer: &Buffer) -> &Vec<String> {
    &buffer.lines
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            file_path: None,
        }
    }

    pub fn load<T: AsRef<str>>(file_path: T) -> Self {
        let file_path = file_path.as_ref().to_string();

        fs::read_to_string(&file_path)
            .map(|content| Self {
                lines: content.lines().map(|s| s.to_string()).collect(),
                file_path: Some(file_path.to_string()),
            })
            .unwrap_or_else(|_| Self {
                lines: vec![],
                file_path: Some(file_path),
            })
    }

    pub fn file_path(&self) -> &Option<String> {
        &self.file_path
    }
}
