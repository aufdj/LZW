use std::collections::HashMap;
use std::fs::{File, metadata};
use std::io::{Write, BufReader, BufWriter, BufRead, Read};
use std::path::Path;
use std::time::Instant;

pub trait BufferedRead {
    fn read_byte(&mut self) -> u8;
    fn read_u16(&mut self) -> u16;
    fn read_byte_checked(&mut self) -> Option<u8>;
    fn read_u16_checked(&mut self) -> Option<u16>;
}
impl BufferedRead for BufReader<File> {
    fn read_byte(&mut self) -> u8 {
        let mut byte = [0u8; 1];
        self.read(&mut byte).unwrap();
        if self.buffer().is_empty() {
            self.consume(self.capacity());
            self.fill_buf().unwrap();
        }
        u8::from_le_bytes(byte)
    }

    fn read_byte_checked(&mut self) -> Option<u8> {
        let mut byte = [0u8; 1];

        let bytes_read = self.read(&mut byte).unwrap();
        if self.buffer().len() <= 0 { 
            self.consume(self.capacity()); 
            self.fill_buf().unwrap();
        }
        if bytes_read == 0 {
            return None;
        }
        Some(u8::from_le_bytes(byte))
    }

    fn read_u16(&mut self) -> u16 {
        let mut bytes = [0u8; 2];
        let len = self.read(&mut bytes).unwrap();
        if self.buffer().is_empty() {
            self.consume(self.capacity());
            self.fill_buf().unwrap();
            if len < 2 {
                self.read_exact(&mut bytes[len..]).unwrap();
            }
        }
        u16::from_le_bytes(bytes)
    }

    fn read_u16_checked(&mut self) -> Option<u16> {
        let mut bytes = [0u8; 2];
        let len = self.read(&mut bytes).unwrap();
        if self.buffer().is_empty() {
            self.consume(self.capacity());
            self.fill_buf().unwrap();
            if len < 2 {
                if self.read_exact(&mut bytes[len..]).is_err() {
                    return None;
                }
            }
        }
        Some(u16::from_le_bytes(bytes))
    }
}

pub trait BufferedWrite {
    fn write_u16(&mut self, output: u16);
}
impl BufferedWrite for BufWriter<File> {
    fn write_u16(&mut self, output: u16) {
        self.write(&output.to_le_bytes()[..]).unwrap();
        if self.buffer().len() >= self.capacity() {
            self.flush().unwrap();
        }
    }
}

pub fn new_input_file(capacity: usize, path: &Path) -> BufReader<File> {
    BufReader::with_capacity(
        capacity, 
        File::open(path).unwrap()
    )
}

pub fn new_output_file(capacity: usize, path: &Path) -> BufWriter<File> {
    BufWriter::with_capacity(
        capacity, 
        File::create(path).unwrap()
    )
}

const MAX_CODE: u16 = 65535;

fn compress(mut file_in: BufReader<File>, mut file_out: BufWriter<File>) {
    let mut eof = false;
    let mut dict_code = 256;

    let mut dict = (0..256)
    .map(|i| (vec![i as u8], i))
    .collect::<HashMap<Vec<u8>, u16>>();
    
    let mut string = vec![file_in.read_byte()]; 

    while !eof {
        while dict.contains_key(&string) {
            if let Some(byte) = file_in.read_byte_checked() {
                string.push(byte); 
            }
            else {
                eof = true;
                break;
            }  
        }
        dict.insert(string.clone(), dict_code); 
        dict_code += 1;

        let last_char = string.pop().unwrap();
        file_out.write_u16(*dict.get(&string).unwrap());

        string.clear();
        string.push(last_char); 

        if dict_code >= MAX_CODE {
            dict_code = 256;
            dict.retain(|_, i| *i < 256);
        }
    }
    if !string.is_empty() {
        file_out.write_u16(*dict.get(&string).unwrap());
    }
    file_out.flush().unwrap();
}

fn decompress(mut file_in: BufReader<File>, mut file_out: BufWriter<File>) {
    let mut dict_code = 256;
    
    let mut dict = (0..256)
    .map(|i| (i, vec![i as u8]))
    .collect::<HashMap<u16, Vec<u8>>>();

    let mut prev_string = Vec::<u8>::with_capacity(64);

    while let Some(code) = file_in.read_u16_checked() {
        if !dict.contains_key(&code) {
            prev_string.push(prev_string[0]);
            dict.insert(code, prev_string);
            dict_code += 1;      
        }
        else if !prev_string.is_empty() {
            prev_string.push((&dict.get(&code).unwrap())[0]);
            dict.insert(dict_code, prev_string);
            dict_code += 1;
        }

        let string = dict.get(&code).unwrap();
        file_out.write(&string).unwrap();

        prev_string = string.to_vec();
        
        if dict_code >= MAX_CODE {
            dict_code = 256;
            dict.retain(|i, _| *i < 256);
        }
    }  
}

fn main() {
    let time = Instant::now();
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let file_in  = new_input_file(4096, Path::new(&args[1]));
    let file_out = new_output_file(4096, Path::new(&args[2]));
    match (&args[0]).as_str() {
        "c" => {
            compress(file_in, file_out);
            println!("Finished compressing.");
        }
        "d" => {
            decompress(file_in, file_out);
            println!("Finished Decompressing.");   
        }
        _ => {
            println!("Enter 'c' to compress and 'd' to decompress.");
        }
    } 
    println!("{} bytes -> {} bytes in {:.2?}", 
        metadata(Path::new(&args[1])).unwrap().len(), 
        metadata(Path::new(&args[2])).unwrap().len(), 
        time.elapsed()
    ); 
}
