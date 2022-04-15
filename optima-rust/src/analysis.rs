use std::{
    fs::File,
    io::{LineWriter, Write},
};
pub trait AsCsvRow {
    fn as_row(&self, i: usize) -> String;
}

pub struct CsvSaver {
    file: File,
    rows: Vec<String>,
    pub header: String,
    pub path: String,
}

impl CsvSaver {
    pub fn save_element(&mut self, row: &dyn AsCsvRow) {
        self.rows.push(row.as_row(self.rows.len()));
    }

    pub fn flush(&mut self) {
        if self.rows.len() > 0 {
            let mut line_writer = LineWriter::new(&self.file);
            line_writer
                .write_all(self.rows.join("\n").as_bytes())
                .expect("Failed to flush changes to csv file");
            self.rows.clear();
        }
    }

    pub fn reset(&mut self, file_path: String, header: Option<String>) {
        self.path = file_path;
        if header.is_some() {
            self.header = header.unwrap() + "\n";
        }
        self.file = File::create(&self.path).expect("Cannot create csv file");
        self.file
            .write_all(self.header.as_bytes())
            .expect("Failed to write a csv header");
        self.rows.clear();
    }
}

impl CsvSaver {
    pub fn new(file_path: String, header: String) -> Self {
        let mut file = File::create(&file_path).expect("Cannot create csv file");
        file.write_all(header.as_bytes())
            .expect("Failed to write a csv header");
        Self {
            file,
            header: header + "\n",
            path: file_path,
            rows: vec![],
        }
    }
}
