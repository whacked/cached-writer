use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

use log::{info, Level};

pub struct CachedWriter {
    path: String,
    buffer: Vec<String>,
    flush_frequency: u16,
}

impl CachedWriter {
    pub fn new(path: String, flush_frequency: u16) -> Self {
        CachedWriter {
            path,
            buffer: Vec::new(),
            flush_frequency,
        }
    }

    pub fn append(&mut self, data: String) {
        self.buffer.push(data);
        if self.buffer.len() as u16 > self.flush_frequency {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        let mut file = OpenOptions::new()
            .create(true) // Create the file if it doesn't exist
            .append(true)
            .open(&self.path)
            .expect(&format!("Failed to open file {}", self.path));

        // Create a buffered writer for better I/O performance
        let mut writer = BufWriter::new(file);

        // Iterate over the lines and write them to the file
        for line in &self.buffer {
            write!(writer, "\n{}", line).expect("Failed to write to file");
        }

        // Flush the buffered writer to ensure all data is written
        writer
            .flush()
            .expect(&format!("Failed to flush writer for file {}", self.path));

        println!("[WRITER] flushed {} records to {}", self.buffer.len(), self.path);
        self.buffer.clear();
    }
}

fn main() {
    // Usage example
    let mut writer = CachedWriter::new("example-file.txt".to_string(), 100);
    writer.append("Data 1".to_string());
    writer.append("Data 2".to_string());
    writer.flush();
}
