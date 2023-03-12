use std::cmp::min;
use std::fs::File;
use crate::printable_error::PrintableError;

use quick_drop_deque::QuickDropDeque;
use crate::runtime::columns::lazily_split_line::LazilySplitLine;
use crate::util::index_in_full_dq;

#[allow(dead_code)]
struct FileWithPath {
    path: String,
    file: File,
}

pub struct FileReader {
    file: Option<FileWithPath>,
    slop: QuickDropDeque,
    rs: Vec<u8>,
    next_rs: Option<Vec<u8>>,
    end_of_current_record: usize,
    line: LazilySplitLine,
}

impl FileReader {
    pub fn new() -> Self {
        Self {
            slop: QuickDropDeque::with_io_size(16*1024, 8*1024),
            file: None,
            rs: vec![10], //space
            next_rs: None,
            line: LazilySplitLine::new(),
            end_of_current_record: 0,
        }
    }

    pub fn next_file(&mut self, file: File, path: String) {
        self.file = Some(FileWithPath { file, path })
    }

    pub fn try_next_record(&mut self) -> Result<bool, PrintableError> {


        self.line.next_record();
        let file = if let Some(file) = &mut self.file {
            file
        } else {
            return Ok(false);
        };


        // Drop last record if any
        self.slop.drop_front(self.end_of_current_record);


        // Regardless of whether RS has changed drop the old RS from the
        let mut rs_idx = index_in_full_dq(&self.rs, &self.slop);
        if rs_idx == Some(0) {
            // If the deque starts with RS drop it. If not keep the value around and we won't
            // redo the search in the read loop below
            self.slop.drop_front(self.rs.len());
            rs_idx = None;
        }

        if let Some(next_rs) = self.next_rs.take() {
            // If rs changes we need to wipe out rs_idx.
            self.rs = next_rs;
            rs_idx = None;
        }

        loop {
            // Check if our last read grabbed more than 1 record
            if let Some(idx) = rs_idx.or_else(|| index_in_full_dq(&self.rs, &self.slop)) {
                self.end_of_current_record = idx;
                return Ok(true);
            }
            // If not then read some bytes into buf then copy to slop
            let bytes_read = match self.slop.read(&mut file.file) {
                Ok(b) => b,
                Err(err) => return Err(PrintableError::new(format!("Something went wrong reading from file `{}`. Error: {}", &file.path, err))),
            };

            if bytes_read == 0 {
                // No more data!
                self.end_of_current_record = self.slop.len();

                if self.slop.len() != 0 {
                    // Reached EOF but we have slop from last read without RS completing it
                    return Ok(true);
                } else {
                    // Reached EOF and nothing left in slop buffer we're out of records
                    return Ok(false);
                }
            }
        }
    }

    pub fn get_into_buf(&mut self, idx: usize, result: &mut Vec<u8>) {
        if idx == 0 {
            let slices = self.slop.as_slices();
            let bytes_to_move = self.end_of_current_record;
            let elements_from_left = min(slices.0.len(), bytes_to_move);
            result.extend_from_slice(&slices.0[0..elements_from_left]);
            if elements_from_left < bytes_to_move {
                let remaining = bytes_to_move - elements_from_left;
                result.extend_from_slice(&slices.1[0..remaining]);
            }
        } else {
            self.line.get_into(&self.slop, idx, self.end_of_current_record, result);
        }
    }

    pub fn get(&mut self, idx: usize) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::with_capacity(self.end_of_current_record);
        if self.end_of_current_record != 0 {
            self.get_into_buf(idx, &mut result);
        }
        result
    }

    pub fn set_rs(&mut self, rs: Vec<u8>) {
        if rs == self.rs {
            return
        }
        self.next_rs = Some(rs);
    }
    pub fn get_rs(&mut self) -> &[u8] {
        &self.rs
    }
    pub fn set_fs(&mut self, bytes: Vec<u8>) {
        self.line.set_field_sep(bytes)
    }
    pub fn get_fs(&mut self) -> &[u8] {
        self.line.get_field_sep()
    }
}
