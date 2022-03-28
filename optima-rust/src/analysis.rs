use std::{
    fs::File,
    io::{LineWriter, Write},
};

pub trait Saver<T> {
    fn save_element(&mut self, to_save: &T);
    fn flush(&mut self);
    fn reset(&mut self);
}

pub struct CsvSaver {
    file: File,
    rows: Vec<String>,
    pub header: String,
    pub path: String,
}

impl<'a, T> Saver<T> for CsvSaver
where
    T: AsCsvRow,
{
    fn save_element(&mut self, row: &T) {
        self.rows.push(row.as_row(self.rows.len()));
    }

    fn flush(&mut self) {
        if self.rows.len() > 0 {
            let mut line_writer = LineWriter::new(&self.file);
            line_writer
                .write_all(self.rows.join("\n").as_bytes())
                .expect("Failed to flush changes to csv file");
            self.rows.clear();
        }
    }

    fn reset(&mut self) {
        self.file = File::create(&self.path).expect("Cannot create csv file");
        self.file
            .write_all(self.header.as_bytes())
            .expect("Failed to write a csv header");
        self.rows.clear();
    }
}

pub trait AsCsvRow {
    fn as_row(&self, i: usize) -> String;
}

impl<'a> CsvSaver {
    pub fn new(file_path: String, header: String) -> Self {
        let mut file = File::create(&file_path).expect("Cannot create csv file");
        file.write_all(header.as_bytes())
            .expect("Failed to write a csv header");
        Self {
            file,
            header,
            path: file_path,
            rows: vec![],
        }
    }
}
