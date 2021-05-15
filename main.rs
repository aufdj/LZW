use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::env::args;

fn compress(mut file_in: File, mut file_out: File) {
    let mut s = String::new();
    file_in.read_to_string(&mut s).expect("Couldn't read file to string.");
    // max code value set to 55295 because values 55296 - 55551 are reserved for utf-16
    // and invalid in utf-8
    let max_code: u16 = 55295;
    let mut dictionary: HashMap<String, u16> = HashMap::new();

    for i in 0..256 { 
        dictionary.insert((i as u8 as char).to_string(), i); 
    }

    let mut i = 1;
    let mut dictionary_code = 257;
    let mut current_string = String::new(); 

    current_string.push_str(&s[0..1]); 

    loop {
        while dictionary.contains_key(&current_string) {
            current_string.push_str(&s[i..i+1]);         
            i+=1;
        }
        //println!("Insert {} to dictionary with code {}", &current_string, &dictionary_code);
        if dictionary_code <= max_code {
            dictionary.insert(current_string.clone().to_string(), dictionary_code); 
            dictionary_code+=1;
        }
        
        let last_char = current_string.pop().expect("Couldn't pop.");
        //println!("Output {:?}", &(dictionary.get(&current_string).expect("Couldn't get key.")).to_le_bytes());
        file_out.write(&(dictionary.get(&current_string).expect("Couldn't get key."))
                         .to_le_bytes())                .expect("Couldn't write to file.");

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
        file_out.write(&(dictionary.get(&current_string).expect("Couldn't get key."))
                         .to_le_bytes())                .expect("Couldn't write to file.");
    }   
    println!("Finished compressing.");
}
fn decompress(mut file_in: File, mut file_out: File) {
    let mut s = String::new();
    file_in.read_to_string(&mut s).expect("Couldn't read file to string.");
    let mut chars = s.chars();

    let max_code: u16 = 55295;
    let mut dictionary: HashMap<u16, String> = HashMap::new();

    for i in 0..256 {
        dictionary.insert(i, (i as u8 as char).to_string());
    }
   
    let mut compressed_code: u16;
    let mut dictionary_code = 257;
    let mut previous_string = String::new();

    loop {
        compressed_code = chars.next().expect("Finished decompressing.") as u16;
        if !dictionary.contains_key(&compressed_code) && dictionary_code < max_code {
            //println!("Didn't recognize code: {} (compressed)", &compressed_code);
            //println!("Insert {} to dictionary with code {}", previous_string.clone().to_owned() + &previous_string[0..1], &compressed_code);
            dictionary.insert(compressed_code, previous_string.clone().to_owned() + &previous_string[0..1]);
            dictionary_code+=1;      
        }
        else if !previous_string.is_empty() && dictionary_code < max_code {
            //println!("Insert {} to dictionary with code {}", previous_string.clone().to_owned() + &(&dictionary.get(&compressed_code).expect("Couldn't get key"))[0..1], &dictionary_code);
            dictionary.insert(dictionary_code, previous_string.clone().to_string() + &(&dictionary.get(&compressed_code).expect("Couldn't get key"))[0..1]);
            dictionary_code+=1;
        }
        //println!("Compressed code: {}, String: {}", &compressed_code, &dictionary.get(&compressed_code).unwrap());
        file_out.write(&(dictionary.get(&compressed_code).expect("Couldn't get key."))
                        .as_bytes())                     .expect("Couldn't write to file.");

        previous_string = dictionary.get(&compressed_code).expect("Couldn't get key.").to_string();
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



