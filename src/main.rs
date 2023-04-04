use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufReader, BufWriter, BufRead, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Instant;

// Indicates an empty or non-empty buffer. 
#[derive(PartialEq, Eq)]
pub enum BufferState {
    NotEmpty,
    Empty,
}

/// A trait for handling buffered reading.
pub trait BufferedRead {
    fn read_byte(&mut self) -> u8;
    fn read_u16(&mut self) -> u16;
}
impl BufferedRead for BufReader<File> {
    /// Read one byte from an input file.
    fn read_byte(&mut self) -> u8 {
        let mut byte = [0u8; 1];
        match self.read(&mut byte) {
            Ok(_)  => {},
            Err(e) => {
                println!("Function read_byte failed.");
                println!("Error: {}", e);
            },
        };
        if self.buffer().is_empty() {
            self.consume(self.capacity());
            match self.fill_buf() {
                Ok(_)  => {},
                Err(e) => {
                    println!("Function read_byte failed.");
                    println!("Error: {}", e);
                },
            }
        }
        u8::from_le_bytes(byte)
    }
    fn read_u16(&mut self) -> u16 {
        let mut bytes = [0u8; 2];
        let len = match self.read(&mut bytes) {
            Ok(len)  => { len },
            Err(e) => {
                println!("Function read_u16 failed.");
                println!("Error: {}", e);
                0
            },
        };
        if self.buffer().is_empty() {
            self.consume(self.capacity());
            match self.fill_buf() {
                Ok(_)  => {},
                Err(e) => {
                    println!("Function read_u16 failed.");
                    println!("Error: {}", e);
                },
            }
            if len < 4 {
                self.read_exact(&mut bytes[len..]).unwrap();
            }
        }
        u16::from_le_bytes(bytes)
    }
}

/// A trait for handling buffered writing.
pub trait BufferedWrite {
    fn write_u16(&mut self, output: u16);
}
impl BufferedWrite for BufWriter<File> {
    fn write_u16(&mut self, output: u16) {
        match self.write(&output.to_le_bytes()[..]) {
            Ok(_)  => {},
            Err(e) => {
                println!("Function write_u16 failed.");
                println!("Error: {}", e);
            },
        }
        if self.buffer().len() >= self.capacity() {
            match self.flush() {
                Ok(_)  => {},
                Err(e) => {
                    println!("Function write_u16 failed.");
                    println!("Error: {}", e);
                },
            }
        }
    }
}


/// Takes a file path and returns an input file wrapped in a BufReader.
pub fn new_input_file(capacity: usize, path: &Path) -> BufReader<File> {
    BufReader::with_capacity(
        capacity, 
        File::open(path).unwrap()
    )
}

/// Takes a file path and returns an output file wrapped in a BufWriter.
pub fn new_output_file(capacity: usize, path: &Path) -> BufWriter<File> {
    BufWriter::with_capacity(
        capacity, 
        File::create(path).unwrap()
    )
}

fn file_len(path: &Path) -> u64 {
    path.metadata().unwrap().len()
}

fn compress(mut file_in: BufReader<File>, mut file_out: BufWriter<File>) {
    let size_in = file_in.seek(SeekFrom::End(0)).unwrap();
    file_in.seek(SeekFrom::Start(0)).unwrap();

    let max_code: u16 = 65535;
    let mut dict: HashMap<Vec<u8>, u16> = HashMap::new();

    let mut dict_code = 257;
    for i in 0..256 { 
        dict.insert(vec![i as u8], i); 
    }
    
    
    let mut curr_string: Vec<u8> = vec![file_in.read_byte()]; 

    let mut file_pos = 1;

    loop {
        while dict.contains_key(&curr_string) {
            curr_string.push(file_in.read_byte()); 
            file_pos += 1; 
            if file_pos >= size_in { break; }   
        }
        if dict_code <= max_code {
            dict.insert(curr_string.clone(), dict_code); 
            dict_code+=1;
        }

        let last_char = curr_string.pop().unwrap();
        let code = *dict.get(&curr_string).unwrap();
        file_out.write_u16(code);

        curr_string.clear();
        curr_string.push(last_char); 

        if dict_code >= max_code {
            dict_code = 257;
            dict.clear(); 
            for i in 0..256 {
                dict.insert(vec![i as u8], i);
            }
        }
        if file_pos >= size_in { break; } 
    }
    if !curr_string.is_empty() {
        let code = *dict.get(&curr_string).unwrap();
        file_out.write_u16(code);
        file_out.flush().unwrap();
    } 
    println!("Finished compressing.");
}

fn decompress(mut file_in: BufReader<File>, mut file_out: BufWriter<File>) {
    let size_in = file_in.seek(SeekFrom::End(0)).unwrap();
    file_in.seek(SeekFrom::Start(0)).unwrap();

    let max_code: u16 = 65535;
    let mut dict: HashMap<u16, Vec<u8>> = HashMap::new();

    let mut dict_code = 257;
    for i in 0..256 {
        dict.insert(i, vec![i as u8]);
    }

    let mut prev_string: Vec<u8> = Vec::with_capacity(64);

    let mut file_pos = 0;

    loop { 
        let code = file_in.read_u16();
        file_pos += 2;

        if !dict.contains_key(&code) && dict_code < max_code { // Didn't recognize code
            prev_string.push(prev_string[0]);
            
            dict.insert(code, prev_string);
            dict_code += 1;      
        }
        else if !prev_string.is_empty() && dict_code < max_code {
            prev_string.push((&dict.get(&code).unwrap())[0]);
            
            dict.insert(dict_code, prev_string);
            dict_code += 1;
        }

        let string = dict.get(&code).unwrap();
        file_out.write(&string).unwrap();

        prev_string = string.to_vec();
        
        if dict_code >= max_code {
            dict_code = 257;
            dict.clear(); 
            for i in 0..256 {
                dict.insert(i, vec![i as u8]);
            }
        }
        if file_pos >= size_in { break; }
    } 
    println!("Finished Decompressing.");    
}

fn main() {
    let time = Instant::now();
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let file_in  = new_input_file(4096, Path::new(&args[1]));
    let file_out = new_output_file(4096, Path::new(&args[2]));
    match (&args[0]).as_str() {
        "c" => compress(file_in, file_out),
        "d" => decompress(file_in, file_out),
        _ => println!("Enter 'c' to compress and 'd' to decompress."),
    } 

    let size_in  = file_len(Path::new(&args[1]));
    let size_out = file_len(Path::new(&args[2]));
    println!("{} bytes -> {} bytes in {:.2?}", 
        size_in, size_out, time.elapsed()); 
}
