use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufReader, BufWriter, BufRead};
use std::env::args;
use std::str::from_utf8;
use std::process;

fn compress(file_in: File, file_out: File) {
    let mut buf_in = BufReader::with_capacity(1024, file_in);
    let mut buf_out = BufWriter::with_capacity(1024, file_out);
    let mut writes = 0; // keeps track of number of bytes written to output
    buf_in.fill_buf().unwrap();

    let max_code: u16 = 65535;
    let mut dictionary: HashMap<String, u16> = HashMap::new();

    for i in 0..256 { 
        dictionary.insert((i as u8 as char).to_string(), i); 
    }

    let mut i = 1;
    let mut dictionary_code = 257;
    let mut current_string = String::new(); 

    current_string.push_str(&from_utf8(&buf_in.buffer()).unwrap()[0..1]); 

    loop {
        while dictionary.contains_key(&current_string) {
            current_string.push_str(&from_utf8(&buf_in.buffer()).unwrap()[i..i+1]);         
            i+=1;
            if i >= buf_in.capacity() {
                //println!("Refilling buffer.");
                buf_in.consume(buf_in.capacity());
                buf_in.fill_buf().unwrap();
                i = 0;
            }
        }
        //println!("Insert {} to dictionary with code {}", &current_string, &dictionary_code);
        if dictionary_code <= max_code {
            dictionary.insert(current_string.clone().to_string(), dictionary_code); 
            dictionary_code+=1;
        }
        
        let last_char = current_string.pop().unwrap();
        //println!("Output {:?}", &(dictionary.get(&current_string).expect("Couldn't get key.")).to_le_bytes()[0..1]);
        //println!("Output {:?}", &(dictionary.get(&current_string).expect("Couldn't get key.")).to_le_bytes()[1..2]);
        buf_out.write(&(dictionary.get(&current_string).unwrap()).to_le_bytes()).unwrap();
        //buf_out.write(&(dictionary.get(&current_string).unwrap()).to_le_bytes()[1..2]).unwrap();
        writes+=2; // 2 bytes written on each write
        if writes >= buf_out.capacity() {
            buf_out.flush().unwrap();
        }
        current_string = last_char.to_string(); 
        //println!("Root string set to {}", &current_string);
        if last_char == '#' { break; }

        if dictionary_code >= max_code {
            //println!("Resetting dictionary.");
            dictionary_code = 257;
            dictionary.clear(); 
            for i in 0..256 {
                dictionary.insert((i as u8 as char).to_string(), i);
            }
        }
    }
    if !current_string.is_empty() {
        //println!("Writing EOF code.");
        buf_out.write(&(dictionary.get(&current_string).unwrap()).to_le_bytes()).unwrap();
        buf_out.flush().unwrap();
    }   
    println!("Finished compressing.");
}
fn decompress(file_in: File, file_out: File) {
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

    loop {
        let mut compressed_code: u16 = 0;
        compressed_code += *((buf_in.buffer()[buf_pos..buf_pos+1]).get(0).unwrap()) as u16;
        compressed_code += (*((buf_in.buffer()[buf_pos+1..buf_pos+2]).get(0).unwrap()) as u16) * 256;
        buf_pos+=2;
        if buf_pos >= buf_in.buffer().len() {
            buf_in.consume(buf_in.capacity());
            buf_in.fill_buf().unwrap();
            buf_pos = 0;
        }
        if buf_in.buffer().is_empty() {
            println!("Finished Decompressing.");
            process::exit(0);
        }
        if !dictionary.contains_key(&compressed_code) && dictionary_code < max_code {
            //println!("Didn't recognize code: {} (compressed)", &compressed_code);
            //println!("Insert {} to dictionary with code {}", previous_string.clone().to_owned() + &previous_string[0..1], &compressed_code);
            dictionary.insert(compressed_code, previous_string.clone().to_owned() + &previous_string[0..1]);
            dictionary_code+=1;      
        }
        else if !previous_string.is_empty() && dictionary_code < max_code {
            //println!("Insert {} to dictionary with code {}", previous_string.clone().to_owned() + &(&dictionary.get(&compressed_code).expect("Couldn't get key"))[0..1], &dictionary_code);
            dictionary.insert(dictionary_code, previous_string.clone().to_string() + &(&dictionary.get(&compressed_code).unwrap())[0..1]);
            dictionary_code+=1;
        }
        //println!("Compressed code: {}, String: {}", &compressed_code, &dictionary.get(&compressed_code).unwrap());
        buf_out.write(&(dictionary.get(&compressed_code).unwrap()).as_bytes()).unwrap();
        writes += dictionary.get(&compressed_code).unwrap().as_bytes().len();
        if writes >= buf_out.capacity() - 50 { // each write is a variable number of bytes, so flush the buffer early to avoid overflowing
            buf_out.flush().unwrap();
        }
        previous_string = dictionary.get(&compressed_code).unwrap().to_string();
        //println!("Set previous string to {}", &previous_string);

        if dictionary_code >= max_code {
            //println!("Resetting dictionary.");
            dictionary_code = 257;
            dictionary.clear(); 
            for i in 0..256 {
                dictionary.insert(i, (i as u8 as char).to_string());
            }
        }
    }      
}
fn main() {
    let args: Vec<String> = args().collect();
    let file_in = File::open(&args[2]).expect("Couldn't open input file.");
    let file_out = File::create(&args[3]).expect("Couldn't open output file.");
    match (&args[1]).as_str() {
        "c" => compress(file_in, file_out),
        "d" => decompress(file_in, file_out),
        _ => println!("Enter 'c' to compress and 'd' to decompress.")
    }   
}



