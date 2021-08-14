use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufReader, BufWriter, BufRead};
use std::env::args;
use std::str::from_utf8;
use std::fs::metadata;
use std::path::Path;

fn compress(file_in: File, file_out: File, file_size: u64) {
    let mut buf_in = BufReader::with_capacity(1024, file_in);
    let mut buf_out = BufWriter::with_capacity(1024, file_out);
    let mut writes = 0; // keeps track of number of bytes written to output
    buf_in.fill_buf().unwrap();

    let max_code: u16 = 65535;
    let mut dictionary: HashMap<String, u16> = HashMap::new();

    for i in 0..256 { 
        dictionary.insert((i as u8 as char).to_string(), i); 
    }
    
    let mut dictionary_code = 257;
    let mut current_string = String::new(); 

    let mut buf_pos = 1;
    let mut file_pos = 1;

    current_string.push_str(&from_utf8(&buf_in.buffer()).unwrap()[0..1]); 

    loop {
        while dictionary.contains_key(&current_string) {
            current_string.push_str(&from_utf8(&buf_in.buffer()).unwrap()[buf_pos..buf_pos+1]); 
            file_pos+=1;
            if file_pos >= file_size { break; }     
            buf_pos+=1;
            if buf_pos >= buf_in.capacity() {
                buf_in.consume(buf_in.capacity());
                buf_in.fill_buf().unwrap();
                buf_pos = 0;
            }
        }
        if dictionary_code <= max_code {
            dictionary.insert(current_string.clone().to_string(), dictionary_code); 
            dictionary_code+=1;
        }
        
        let last_char = current_string.pop().unwrap();
        buf_out.write(&(dictionary.get(&current_string).unwrap()).to_le_bytes()).unwrap();
        writes+=2; // 2 bytes written on each write
        if writes >= buf_out.capacity() {
            buf_out.flush().unwrap();
        }
        current_string = last_char.to_string(); 

        if dictionary_code >= max_code {
            dictionary_code = 257;
            dictionary.clear(); 
            for i in 0..256 {
                dictionary.insert((i as u8 as char).to_string(), i);
            }
        }
        if file_pos >= file_size { break; } 
    }
    if !current_string.is_empty() {
        buf_out.write(&(dictionary.get(&current_string).unwrap()).to_le_bytes()).unwrap();
        buf_out.flush().unwrap();
    } 
    println!("Finished compressing.");
}
fn decompress(file_in: File, file_out: File, file_size: u64) {
    let mut buf_in = BufReader::with_capacity(1024, file_in);
    let mut buf_out = BufWriter::with_capacity(1024, file_out);
    let mut writes = 0;
    buf_in.fill_buf().unwrap();

    let max_code: u16 = 65535;
    let mut dictionary: HashMap<u16, String> = HashMap::new();

    for i in 0..256 {
        dictionary.insert(i, (i as u8 as char).to_string());
    }

    let mut dictionary_code = 257;
    let mut previous_string = String::new();

    let mut buf_pos = 0;
    let mut file_pos = 0;

    'outer: loop {
        let mut compressed_code: u16 = 0;
        compressed_code += *((buf_in.buffer()[buf_pos..buf_pos+1]).get(0).unwrap()) as u16;
        compressed_code += (*((buf_in.buffer()[buf_pos+1..buf_pos+2]).get(0).unwrap()) as u16) * 256;
        if compressed_code == 0 { break; }
        buf_pos+=2;
        file_pos+=2;
        if buf_pos >= buf_in.buffer().len() {
            buf_in.consume(buf_in.capacity());
            buf_in.fill_buf().unwrap();
            buf_pos = 0;
        }

        if !dictionary.contains_key(&compressed_code) && dictionary_code < max_code { // didn't recognize code
            dictionary.insert(compressed_code, previous_string.clone().to_owned() + &previous_string[0..1]);
            dictionary_code+=1;      
        }
        else if !previous_string.is_empty() && dictionary_code < max_code {
            dictionary.insert(dictionary_code, previous_string.clone().to_string() + &(&dictionary.get(&compressed_code).unwrap())[0..1]);
            dictionary_code+=1;
        }

        buf_out.write(&(dictionary.get(&compressed_code).unwrap()).as_bytes()).unwrap();
        writes += dictionary.get(&compressed_code).unwrap().as_bytes().len();
        if writes >= buf_out.capacity() - 50 { // each write is a variable number of bytes, so flush the buffer early to avoid overflowing
            buf_out.flush().unwrap();
        }
        previous_string = dictionary.get(&compressed_code).unwrap().to_string();

        if dictionary_code >= max_code {
            dictionary_code = 257;
            dictionary.clear(); 
            for i in 0..256 {
                dictionary.insert(i, (i as u8 as char).to_string());
            }
        }
        if file_pos >= file_size { break 'outer; }
    } 
    println!("Finished Decompressing.");     
}
fn main() {
    let args: Vec<String> = args().collect();
    let file_in = File::open(&args[2]).expect("Couldn't open input file.");
    let file_out = File::create(&args[3]).expect("Couldn't open output file.");
    let file_size = metadata(Path::new(&args[2])).unwrap().len();
    match (&args[1]).as_str() {
        "c" => compress(file_in, file_out, file_size),
        "d" => decompress(file_in, file_out, file_size),
        _ => println!("Enter 'c' to compress and 'd' to decompress.")
    }   
}



