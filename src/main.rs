mod rplace;
use std::{time::Instant, fs::File, io::Write, ops::Index};

use memmap::Mmap;
use serde::{Serialize, Deserialize};
use speedy2d::Window;
use rplace::{window::RedditPlaceWindowHandler, data::RPlaceDataReader, dataset::RPlaceDataset};
use strum::IntoEnumIterator;

use crate::rplace::{dataset::RPlaceDatasetDatapoint, pixel::PixelColor, data};

pub struct RPlaceDataCounter {
    pub counter: Vec<Vec<u32>>,
}

impl RPlaceDataCounter {
    pub fn empty(size: usize) -> RPlaceDataCounter {
        let mut counter = Vec::new();
        for y in 0..size {
            let mut row = Vec::new();
            for x in 0..size {
                row.push(0);
            }
            counter.push(row);
        }

        RPlaceDataCounter { 
            counter,
        }
    }

    pub fn increment(&mut self, x: usize, y: usize) {
        self.counter[y][x] += 1;
    }
}

fn run_visualizer(file_path: &str, size: usize) {
    let window = Window::new_fullscreen_borderless("R/Place Renderer").unwrap();
    window.run_loop(RedditPlaceWindowHandler::new(file_path, size));
}

fn iterate_data(file_path: &str) {
    println!("Iterating data");

    let size = 2000;
    let mut history = 0;
    let limit = 170_000_000;
    let mut start_time = Instant::now();
    let mut found_day2 = false;
    let mut found_day3 = false;
    //let mut dataset = RPlaceDataset::empty(size);
    let mut counter = RPlaceDataCounter::empty(size);
    if let Some(reader) = RPlaceDataReader::new(file_path) {
        let iterator = reader.into_iter();
        for datapoint in iterator.take(limit) {
            if history % 1000000 == 0 {
                let duration = start_time.elapsed();
                println!("Start | At {} lines | Duration - {:?}", history, duration);
                start_time = Instant::now();
            }

            if !found_day2 && datapoint.coordinate.x >= 1000.0 {
                found_day2 = true;
                println!("Day 2 | At {} lines | {:?}", history, datapoint);
            }

            if !found_day3 && datapoint.coordinate.y >= 1000.0 {
                found_day3 = true;
                println!("Day 3 | At {} lines | {:?}", history, datapoint);
            }

            //dataset.add(datapoint, datapoint.coordinate.x as usize, datapoint.coordinate.y as usize);
            counter.increment(datapoint.coordinate.x as usize, datapoint.coordinate.y as usize);
            history += 1;
        }
    }

    let mut all_counts: Vec<&u32> = counter.counter.iter().flatten().collect::<Vec<&u32>>();
    all_counts.sort_by(|&&a, &&b| b.cmp(&a));

    for (i, count) in all_counts.iter().enumerate().take(100) {
        println!("Most counts: {} {}", i, count);
    }
    //println!("Sleeping - History size: {}", history);
    //thread::sleep(Duration::from_secs(30));
}

fn test_mmap() {
    let mut datapoints = Vec::new();
    for (i, color) in PixelColor::iter().enumerate() {
        let datapoint = RPlaceDatasetDatapoint {
            timestamp: i as u64,
            user_id: i as u32,
            color, 
            is_mod: (i % 2) != 0,
        };
        datapoints.push(datapoint);
    }

    let length = datapoints.len();
    let bytes: Vec<u8> = datapoints.iter().map(|x: &RPlaceDatasetDatapoint| x.to_bytes()).flatten().collect::<Vec<u8>>();
    let size = bytes.len() / datapoints.len();
    println!("Hello, world! | datapoints {} | size {} | bytes {} | {:?}", length, size, bytes.len(), bytes);

    {
        let mut file = File::create("temp").unwrap();
        file.write_all(bytes.as_slice()).unwrap();
        file.flush().unwrap();
    }

    let file = File::open("temp").unwrap();
    let mmap1 = unsafe { Mmap::map(&file).unwrap() };
    let mmap2 = unsafe { Mmap::map(&file).unwrap() };

    for i in 0..length/2 {
        let data = mmap1.get(i*size..(i+1)*size).unwrap();
        println!("Data {} | {:?} | {:?}", i, data, RPlaceDatasetDatapoint::from_bytes(data));
    }
    
    for i in length/2..length {
        let data = mmap2.get(i*size..(i+1)*size).unwrap();
        println!("Data {} | {:?} | {:?}", i, data, RPlaceDatasetDatapoint::from_bytes(data));
    }

    for i in 0..length/2 {
        let data = mmap2.get(i*size..(i+1)*size).unwrap();
        println!("Data {} | {:?} | {:?}", i, data, RPlaceDatasetDatapoint::from_bytes(data));
    }

    for i in length/2..length {
        let data = mmap1.get(i*size..(i+1)*size).unwrap();
        println!("Data {} | {:?} | {:?}", i, data, RPlaceDatasetDatapoint::from_bytes(data));
    }

    println!("Done");
}

#[derive(Serialize, Deserialize, Debug)]
struct CompressedDatasetMetadata {
    canvas_width: u32,
    canvas_height: u32,
    min_timestamp: u64,
    max_timestamp: u64,
    datapoint_size: u8,
    history_metadata: Vec<CompressedDatapointHistoryMetadata>,
}

impl CompressedDatasetMetadata {
    fn new(canvas_size: usize) -> CompressedDatasetMetadata {
        let default_metadata = CompressedDatapointHistoryMetadata::default();
        CompressedDatasetMetadata {
            canvas_width: canvas_size as u32,
            canvas_height: canvas_size as u32,
            min_timestamp: u64::MIN,
            max_timestamp: u64::MAX,
            datapoint_size: RPlaceDatasetDatapoint::start().to_bytes().len() as u8,
            history_metadata: vec![default_metadata; canvas_size * canvas_size],
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn compressed_size(canvas_size: usize) -> u64 {
        let metadata = CompressedDatasetMetadata::new(canvas_size);
        metadata.to_bytes().len() as u64
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct CompressedDatapointHistoryMetadata {
    offset: u32,
    length: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct CompressedDatasetData {
    data: Vec<RPlaceDatasetDatapoint>
}

#[derive(Serialize, Deserialize, Debug)]
struct CompressedDataset {
    metadata: CompressedDatasetMetadata,
    data: CompressedDatasetData,
}

struct CompressedDatasetMmap {
    mmap: Mmap,
    metadata: CompressedDatasetMetadata,
    data_start_idx: u64,
}

impl CompressedDatasetMmap {
    fn new(file_path: &str) -> CompressedDatasetMmap {
        let file = File::open(file_path).unwrap();
        let mmap = unsafe { Mmap::map(&file).unwrap() };

        let canvas_width: u32 = bincode::deserialize(mmap.get(0..4).unwrap()).unwrap();
        let canvas_height: u32 = bincode::deserialize(mmap.get(0..4).unwrap()).unwrap();
        assert_eq!(canvas_width, canvas_height);

        let data_start_idx = CompressedDatasetMetadata::compressed_size(canvas_width as usize);
        let metadata_bytes = mmap.get(0..data_start_idx as usize).unwrap();
        let metadata = bincode::deserialize(metadata_bytes).unwrap();

        let num_datapoints: u64 = bincode::deserialize(mmap.get(data_start_idx as usize..(data_start_idx+8) as usize).unwrap()).unwrap();
        println!("num datapoints {}", num_datapoints);
        CompressedDatasetMmap { 
            mmap, 
            metadata, 
            data_start_idx,
        }
    }

    fn datapoint_history_bytes(&self, x: u32, y: u32) -> &[u8] {
        let metadata_idx = y * self.metadata.canvas_width + x;
        let datapoint_history_metadata = &self.metadata.history_metadata[metadata_idx as usize];
        let start_idx = self.data_start_idx + 8 + (datapoint_history_metadata.offset as u64 * self.metadata.datapoint_size as u64);
        let end_idx = self.data_start_idx + 8 + ((datapoint_history_metadata.offset + datapoint_history_metadata.length) as u64 * self.metadata.datapoint_size as u64);

        //println!("fetching (x, y)=({} ,{}) from bytes {}..{} | metadata offset {} length {}", x, y, start_idx, end_idx, datapoint_history_metadata.offset, datapoint_history_metadata.length);
        self.mmap.get(start_idx as usize..end_idx as usize).unwrap()
    }
}

struct CompressedDatapointHistoryBytes<'a> {
    bytes: &'a [u8],
    datapoint_size: u8,
    length: usize,
}

impl<'a> CompressedDatapointHistoryBytes<'a> {
    fn new(bytes: &'a [u8]) -> CompressedDatapointHistoryBytes {
        let datapoint_size = RPlaceDatasetDatapoint::compressed_size();
        let length = bytes.len() / datapoint_size as usize;

        // ensures that bytes is properly divisible by length w/ no remainder
        assert_eq!(bytes.len(), datapoint_size as usize * length);

        CompressedDatapointHistoryBytes {
            bytes,
            datapoint_size,
            length,
        }
    }
}

impl<'a> CompressedDatapointHistoryBytes<'a> {
    fn get(&self, index: usize) -> RPlaceDatasetDatapoint {
        let start_idx = index * self.datapoint_size as usize;
        let end_idx = start_idx + self.datapoint_size as usize;
        bincode::deserialize(&self.bytes[start_idx..end_idx]).unwrap()
    }
}

fn write_data_to_file(parquet_dataset_file_path: &str, output_file_path: &str) {
    // day 2 start: 28_201_610
    // day 3 start: 71_784_347
    // end: 160_808_191
    let limit = 160_808_191;
    let print_frequency = 1_000_000;
    let reader = RPlaceDataReader::new(parquet_dataset_file_path).unwrap();
    
    // data to record
    let canvas_size = 2000;
    let mut dataset = RPlaceDataset::new_with_initial_datapoint(canvas_size);
    let mut min_timestamp: u64 = u64::MIN;
    let mut max_timestamp: u64 = u64::MAX;

    let mut count: u64 = 0;
    for (i, record) in reader.into_iter().take(limit).enumerate() {
        if i == 0 {
            println!("Reading datapoint {}: {:?}", i, record);
            
            // set min_timestamp to be one less than the smallest timestamp in the dataset
            min_timestamp = record.timestamp - 1;
            for j in 0..dataset.data.len() {
                for k in 0..dataset.data[j].len() {
                    dataset.data[j][k][0].timestamp = min_timestamp;
                }
            }
        } 

        if i % print_frequency == 0 {
            println!("Reading datapoint {}: {:?}", i, record);
        }
        
        let x = record.coordinate.x as usize;
        let y = record.coordinate.y as usize;
        dataset.add(record.into(), x, y);
        max_timestamp = record.timestamp;
        count += 1;
    }

    println!("Total datapoints: {}", count);

    // create compressed dataset variables 
    let mut compressed_dataset_metadata = CompressedDatasetMetadata {
        canvas_width: canvas_size as u32,
        canvas_height: canvas_size as u32,
        min_timestamp: min_timestamp,
        max_timestamp: max_timestamp,
        datapoint_size: RPlaceDatasetDatapoint::start().to_bytes().len() as u8,
        history_metadata: Vec::new(),
    };

    let mut compressed_dataset_data = CompressedDatasetData {
        data: Vec::new(),
    };

    // populate compressed data values 
    let mut idx = 0u32;
    for (y, row) in dataset.data.iter_mut().enumerate() {
        for (x, history) in row.iter_mut().enumerate() {
            let history_length = history.len() as u32;
            let metadata = CompressedDatapointHistoryMetadata {
                offset: idx,
                length: history_length,
            };

            compressed_dataset_metadata.history_metadata.push(metadata);
            compressed_dataset_data.data.append(history);

            idx += history_length;
        }
    }

    // write data to file
    let compressed_dataset = CompressedDataset {
        metadata: compressed_dataset_metadata,
        data: compressed_dataset_data,
    };
    let dataset_bytes = bincode::serialize(&compressed_dataset).unwrap();
    
    let mut file = File::create(output_file_path).unwrap();
    file.write_all(dataset_bytes.as_slice()).unwrap();
    file.flush().unwrap();

    let metadata_bytes = bincode::serialize(&compressed_dataset.metadata).unwrap();
    let data_bytes = bincode::serialize(&compressed_dataset.data).unwrap();
    let array_bytes = bincode::serialize(&compressed_dataset.data.data).unwrap();
    println!("Length metadata {} | metadata_calculated {} | data {} | vec {} | vec_len {} | dataset {}", metadata_bytes.len(), CompressedDatasetMetadata::compressed_size(canvas_size), data_bytes.len(), array_bytes.len(), compressed_dataset.data.data.len(), dataset_bytes.len());
}

fn read_data_from_compressed_file(file_path: &str) {
    let compressed_dataset_mmap = CompressedDatasetMmap::new(file_path);
    let metadata = &compressed_dataset_mmap.metadata;

    println!("canvas size {} {} | datapoint size {} | min/max timestamp {} {} | metadata size {} ", metadata.canvas_width, metadata.canvas_height, metadata.datapoint_size, metadata.min_timestamp, metadata.max_timestamp, metadata.history_metadata.len());

    let mut dataset: Vec<Vec<CompressedDatapointHistoryBytes>> = Vec::new();
    for y in 0..metadata.canvas_height {
        let mut row: Vec<CompressedDatapointHistoryBytes> = Vec::new();
        for x in 0..metadata.canvas_width {
            let bytes = compressed_dataset_mmap.datapoint_history_bytes(x, y);
            let datapoint_history = CompressedDatapointHistoryBytes::new(bytes);
            row.push(datapoint_history);

        }
        dataset.push(row);
    }

    let start_time = Instant::now();
    for (y, row) in dataset.iter().enumerate() {
        for (x, datapoint_history) in row.iter().enumerate() {
            let datapoint: RPlaceDatasetDatapoint = datapoint_history.get(0);
            datapoint;
            //println!("datapoint history (x, y)=({}, {}) | history {:?} | datapoint {:?}", x, y, datapoint_history.bytes, datapoint);
        }
    }

    println!("Total time to read {} values | {:?}", metadata.canvas_width * metadata.canvas_height, start_time.elapsed());
}

fn main() {
    let file_path = "/Users/michaelqian/Projects/rplace/data/parquet/2022_place_deephaven.parquet";
    //run_visualizer(file_path, 2000);
    //iterate_data(file_path);
    //test_mmap();
    let compressed_data_file_path = "output";
    write_data_to_file(file_path, compressed_data_file_path);
    read_data_from_compressed_file(compressed_data_file_path);
}
