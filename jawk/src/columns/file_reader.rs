use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use crate::columns::index_of::{index_in_dq};
use crate::printable_error::PrintableError;

struct FileWithPath {
    path: String,
    file: File
}

pub struct FileReader {
    file: Option<FileWithPath>,
    rs: Vec<u8>,
    slop: VecDeque<u8>,
    read_buf: [u8; 2048],
}

impl FileReader {
    pub fn new() -> Self {
        Self {
            rs: vec![10],
            slop: VecDeque::with_capacity(2048),
            read_buf: [0; 2048],
            file: None,
        }
    }
    pub fn set_rs(&mut self, rs: Vec<u8>) {
        self.rs = rs;
    }

    pub fn next_file(&mut self, file: File, path: String) {
        self.file = Some(FileWithPath { file, path });
        self.slop.clear();
    }

    fn read_into_buf(file: &mut FileWithPath, buf: &mut [u8; 2048]) -> Result<usize, PrintableError> {
        match file.file.read(buf) {
            Ok(bytes_read) => Ok(bytes_read),
            Err(err) => Err(PrintableError::new(format!("Error reading from file {}\n{}", file.path, err)))
        }
    }

    pub fn try_read_record_into_buf(&mut self, dest_buffer: &mut Vec<u8>) -> Result<bool, PrintableError> {
        dest_buffer.clear();


        loop {
            let file = if let Some(file) = &mut self.file {
              file
            } else {
                return Ok(false)
            };

            // Check if our last read grabbed more than 1 record
            if let Some(idx) = index_in_dq(&self.rs, &self.slop) {
                let drain = self.slop.drain(0..idx);
                dest_buffer.extend(drain);
                self.slop.drain(0..self.rs.len()); // Remove the trailing RS
                return Ok(true);
            }

            // Nope, then read some bytes into buf then copy to slop
            let bytes_read = FileReader::read_into_buf(file, &mut self.read_buf)?;

            if bytes_read == 0 {
                // No new data!
                if self.slop.len() != 0 {
                    // Reached EOF but we have slop from last read without RS completing it
                    dest_buffer.extend(self.slop.drain(0..self.slop.len()));
                    return Ok(true);
                } else {
                    // Reached EOF and nothing left in slop buffer we're out of records
                    return Ok(false);
                }
            }

            // Copy bytes we just read into slop, the loop continues
            self.slop.extend(&self.read_buf[0..bytes_read]);
        }
    }
}