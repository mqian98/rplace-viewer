use parquet::{file::reader::{SerializedFileReader}, record::{Field, Row, reader::RowIter}};
use std::{fs::File, fmt};

// obtained parquest file from the following article: https://medium.com/@deephavendatalabs/the-r-place-dataset-bf4b0d70ce72
// Download URL: https://deephaven.io/wp-content/2022_place_deephaven.parquet

#[derive(Copy, Clone, Debug, Default)]
pub struct RPlaceParquetDatapoint {
    pub timestamp: i64,
    pub user_id: i32, 
    pub rgb: u64,
    pub x1: i16,
    pub y1: i16,
    pub x2: i16,
    pub y2: i16,
}

pub struct RPlaceParquetDataReader {
    file_path: String,
    reader: SerializedFileReader<File>,
}

impl fmt::Debug for RPlaceParquetDataReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RPlaceParquetDataIterator")
         .field("file_path", &self.file_path)
         .finish()
    }
}

impl RPlaceParquetDataReader {
    pub fn new(file_path: &str) -> Option<RPlaceParquetDataReader> {
        match File::open(file_path) {
            Ok(file) => {
                if let Ok(reader) = SerializedFileReader::new(file) {
                    return Some(RPlaceParquetDataReader { 
                        file_path: file_path.to_string(), 
                        reader 
                    });
                }
            },
            _ => return None,
        };

        return None;
    }
}

pub struct RPlaceParquetDataIterator<'a> {
    iter: RowIter<'a>,
}

impl<'a> fmt::Debug for RPlaceParquetDataIterator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RPlaceParquetDataIterator")
         .finish()
    }
}

impl Iterator for RPlaceParquetDataIterator<'_> {
    type Item = RPlaceParquetDatapoint;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(row) => Some(self.convert_row(row)),
            _ => None
        }
    }
}

impl IntoIterator for RPlaceParquetDataReader {
    type Item = RPlaceParquetDatapoint;

    type IntoIter = RPlaceParquetDataIterator<'static>;

    fn into_iter(self) -> Self::IntoIter {
        RPlaceParquetDataIterator {
            iter: self.reader.into_iter()
        }
    }
}

impl<'a> RPlaceParquetDataIterator<'a> {
    fn convert_row(&self, row: Row) -> RPlaceParquetDatapoint {
        let mut datapoint: RPlaceParquetDatapoint = RPlaceParquetDatapoint::default();
        for (string, field) in row.get_column_iter() {
            match (string.as_str(), field) {
                ("timestamp", Field::Long(t)) => datapoint.timestamp = *t,
                ("user_id", Field::Int(u)) => datapoint.user_id = *u,
                ("rgb", Field::Int(rgb)) => datapoint.rgb = (*rgb) as u64,
                ("x1", Field::Short(s)) => datapoint.x1 = *s,
                ("y1", Field::Short(s)) => datapoint.y1 = *s,
                ("x2", Field::Short(s)) => datapoint.x2 = *s,
                ("y2", Field::Short(s)) => datapoint.y2 = *s,
                _ => (),
            }
        }
    
        return datapoint;
    }
}

/* 
fn main() {
    let path = Path::new("/Users/michaelqian/Projects/rplace/data/parquet/2022_place_deephaven.parquet");
    if let Ok(file) = File::open(&path) {
        let reader = SerializedFileReader::new(file).unwrap();

        let parquet_metadata = reader.metadata();
        println!("metadata {:?}\n{:#?}", parquet_metadata.num_row_groups(), parquet_metadata.file_metadata().schema());

        let row_group_reader = reader.get_row_group(0).unwrap();
        println!("columns {:?}", row_group_reader.num_columns());

        let schema = parquet_metadata.file_metadata().schema().clone();
        match row_group_reader.get_row_iter(Some(schema)) {
            Ok(iter) => {
                for row in iter.into_iter().take(5) {
                    let datapoint = convert_row(row);
                    println!("{:#?}", datapoint);
                }
            },
            _ => (),
        }
    }
}
*/