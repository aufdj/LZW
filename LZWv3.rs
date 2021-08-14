use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufReader, BufWriter, BufRead};
use std::env::args;
use std::fs::metadata;
use std::path::Path;
use std::time::Instant;

fn compress(file_in: File, file_out: File, args: Vec<String>, start_time: Instant) -> std::io::Result<()> {
    let file_size_in = metadata(Path::new(&args[2]))?.len();
    let mut buf_in = BufReader::with_capacity(1024, file_in);
    let mut buf_out = BufWriter::with_capacity(1024, file_out);
    let mut writes = 0; 
    buf_in.fill_buf()?;

    let max_code: u16 = 65535;
    let mut dictionary: HashMap<Vec<u8>, u16> = HashMap::new();

    for i in 0..256 { 
        dictionary.insert(vec![i as u8], i); 
    }
    
    let mut dictionary_code = 257;
    let mut current_string: Vec<u8> = vec![buf_in.buffer()[0]]; 

    let mut buf_pos = 1;
    let mut file_pos = 1;

    loop {
        while dictionary.contains_key(&current_string) {
            current_string.push(buf_in.buffer()[buf_pos]); 
            file_pos+=1; 
            if file_pos >= file_size_in { break; }   
            buf_pos+=1;
            if buf_pos >= buf_in.capacity() {
                buf_in.consume(buf_in.capacity());
                buf_in.fill_buf()?;
                buf_pos = 0;
            }
        }
        if dictionary_code <= max_code {
            dictionary.insert(current_string.clone(), dictionary_code); 
            dictionary_code+=1;
        }

        let last_char = current_string.pop().unwrap();
        buf_out.write(&(dictionary.get(&current_string).unwrap()).to_le_bytes())?;
        writes+=2; // 2 bytes written on each write
        if writes >= buf_out.capacity() {
            buf_out.flush()?;
        }

        current_string.clear();
        current_string.push(last_char); 

        if dictionary_code >= max_code {
            dictionary_code = 257;
            dictionary.clear(); 
            for i in 0..256 {
                dictionary.insert(vec![i as u8], i);
            }
        }
        if file_pos >= file_size_in { break; } 
    }
    if !current_string.is_empty() {
        buf_out.write(&(dictionary.get(&current_string).unwrap()).to_le_bytes())?;
        buf_out.flush()?;
    } 

    let file_size_out = metadata(Path::new(&args[3]))?.len();
    println!("Finished compressing.");
    println!("{} bytes -> {} bytes in {:.2?}", file_size_in, file_size_out, start_time.elapsed()); 
    Ok(()) 
}
fn decompress(file_in: File, file_out: File, args: Vec<String>, start_time: Instant) -> std::io::Result<()> {
    let file_size_in = metadata(Path::new(&args[2]))?.len();
    let mut buf_in = BufReader::with_capacity(1024, file_in);
    let mut buf_out = BufWriter::with_capacity(1024, file_out);
    let mut writes = 0;
    buf_in.fill_buf()?;

    let max_code: u16 = 65535;
    let mut dictionary: HashMap<u16, Vec<u8>> = HashMap::new();

    for i in 0..256 {
        dictionary.insert(i, vec![i as u8]);
    }

    let mut dictionary_code = 257;
    let mut previous_string: Vec<u8> = Vec::new();

    let mut buf_pos = 0;
    let mut file_pos = 0;

    loop { 
        let mut compressed_code: u16 = 0;
        compressed_code += buf_in.buffer()[buf_pos] as u16;
        compressed_code += (buf_in.buffer()[buf_pos+1] as u16) * 256;
        buf_pos+=2;
        file_pos+=2;
        if buf_pos >= buf_in.buffer().len() {
            buf_in.consume(buf_in.capacity());
            buf_in.fill_buf()?;
            buf_pos = 0;
        }

        if !dictionary.contains_key(&compressed_code) && dictionary_code < max_code { // didn't recognize code
            previous_string.push(previous_string[0]);
            
            dictionary.insert(compressed_code, previous_string);
            dictionary_code+=1;      
        }
        else if !previous_string.is_empty() && dictionary_code < max_code {
            previous_string.push((&dictionary.get(&compressed_code).unwrap())[0]);
            
            dictionary.insert(dictionary_code, previous_string);
            dictionary_code+=1;
        }

        buf_out.write(&(dictionary.get(&compressed_code).unwrap()))?;
        writes += dictionary.get(&compressed_code).unwrap().len();
        if writes >= buf_out.capacity() - 50 { // each write is a variable number of bytes, so flush the buffer early to avoid overflowing
            buf_out.flush()?;
        }
        previous_string = dictionary.get(&compressed_code).unwrap().to_vec();
        
        if dictionary_code >= max_code {
            dictionary_code = 257;
            dictionary.clear(); 
            for i in 0..256 {
                dictionary.insert(i, vec![i as u8]);
            }
        }
        if file_pos >= file_size_in { break; }
    } 

    let file_size_out = metadata(Path::new(&args[3]))?.len();
    println!("Finished Decompressing.");  
    println!("{} bytes -> {} bytes in {:.2?}", file_size_in, file_size_out, start_time.elapsed());  
    Ok(())
}
fn main() -> std::io::Result<()> {
    let start_time = Instant::now();
    let args: Vec<String> = args().collect();
    let file_in = File::open(&args[2])?;
    let file_out = File::create(&args[3])?;
    match (&args[1]).as_str() {
        "c" => compress(file_in, file_out, args, start_time),
        "d" => decompress(file_in, file_out, args, start_time),
        _ => Ok(println!("Enter 'c' to compress and 'd' to decompress."))
    }   
}




