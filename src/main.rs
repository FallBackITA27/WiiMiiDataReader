use std::io::{self, Read, Write};

use lazy_static::lazy_static;
lazy_static! {
    static ref VALIDFILETYPES: std::collections::HashSet<&'static str> = {
        let mut hashset = std::collections::HashSet::new();
        hashset.insert("miigx");
        hashset.insert("mii");
        hashset.insert("mae");
        hashset.insert("rsd");
        hashset.insert("rcd");
        hashset.insert("rkg");
        hashset
    };
}

fn main() {
    println!("---------------------------");
    println!("|  Wii Mii Data Analyzer  |");
    println!("--------------------------");
    println!("by FallBackITA27");
    println!();
    let files: Vec<String> = match std::path::Path::new("./").read_dir() {
        Err(err) => {
            println!("----------- Error reading folder:");
            println!("{err}");
            println!("Force Exiting...");
            std::process::exit(1);
        }
        Ok(val) => val
            .filter_map(|file| {
                if let Ok(file_data) = file {
                    let file_name = file_data.file_name();
                    let file_name = file_name.to_str().unwrap();
                    if let Some(file_ext) = file_name.split('.').last() {
                        if VALIDFILETYPES.contains(file_ext) {
                            return Some(file_name.to_string());
                        }
                    }
                    None
                } else {
                    None
                }
            })
            .collect(),
    };
    println!("----------- Accepted formats:");
    println!("1. miigx, mii, mae, rsd, rcd");
    println!("2. rkg (MKW Ghost Data)");
    println!();

    if files.is_empty() {
        println!("----------- There are no files in the");
        println!("            current working directory");
        println!("            that  match  any  of  the");
        println!("            accepted         formats!");
        std::process::exit(0);
    }

    let width = (files.len().checked_ilog10().unwrap_or(0) + 1) as usize;
    println!("----------- Matching files in current folder:");
    for (index, file_name) in files.clone().into_iter().enumerate() {
        println!("{:0>width$}. {file_name}", index + 1);
    }

    let number_of_file;
    loop {
        println!();
        print!("Pick a number >> ");
        io::stdout().flush().unwrap();
        let mut wannabe_num = String::new();
        if let Err(err) = io::stdin().read_line(&mut wannabe_num) {
            println!();
            println!("Error reading the number, try again.");
            println!(">>> {err}");
            continue;
        }
        wannabe_num = wannabe_num[0..wannabe_num.len() - 1].to_string();
        match wannabe_num.parse::<usize>() {
            Err(err) => {
                println!();
                println!("Error reading the number, try again.");
                println!(">>> {err}");
            }
            Ok(val) => {
                if val < 1 {
                    println!();
                    println!("Number too low!");
                    continue;
                } else if val > files.len() {
                    println!();
                    println!("Number too high!");
                    continue;
                }
                number_of_file = val;
                break;
            }
        }
    }
    println!();

    let file_path = files.get(number_of_file - 1).unwrap();
    let mii_bytes = if file_path.ends_with("rkg") {
        let bytes = match std::fs::File::open(file_path) {
            Ok(v) => v.bytes(),
            Err(e) => {
                println!("{e}");
                std::process::exit(1);
            }
        };

        bytes
            .enumerate()
            .filter(|(idx, _)| *idx >= 0x3C && *idx < 0x86)
            .map(|(_, val)| val.unwrap())
            .collect::<Vec<u8>>()
    } else {
        match std::fs::File::open(file_path) {
            Ok(v) => v.bytes().map(|val| val.unwrap()).collect::<Vec<u8>>(),
            Err(e) => {
                println!("{e}");
                std::process::exit(1);
            }
        }
    };

    let mut mii_bytes = mii_bytes.iter();

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Reading Data:");
    println!("Is Female? {}", (byte1 & 0b01000000) > 0);
    println!(
        "Birthday: {:>02}/{:>02}",
        (byte1 >> 2) & 0b00001111,
        ((byte1 << 6) | (byte2 >> 2)) >> 3
    );
    println!("Favorite Color: {}", (byte2 >> 1) & 0b00001111);
    println!("Is Favorite? {}", (byte2 & 0b00000001) > 0);

    let mut str_bytes = vec![];
    str_bytes.reserve_exact(20);
    let mut last_was_null = false;
    let mut pop_out = 19;
    for index in 0..20 {
        let byte = *mii_bytes.next().unwrap();
        if pop_out != 0 {
            pop_out -= 1;
        }
        if last_was_null && byte == 0 {
            str_bytes.pop();
            break;
        }
        last_was_null = byte == 0;
        if index % 2 == 0 {
            str_bytes.push((byte as u16) << 8);
        } else {
            let last_byte = str_bytes.pop().unwrap();
            str_bytes.push(last_byte | (byte as u16));
        }
    }
    mii_bytes.nth(pop_out);
    println!("Mii Name: \"{}\"", String::from_utf16(&str_bytes).unwrap());
    std::mem::drop(str_bytes);

    println!("Height: {}", mii_bytes.next().unwrap() & 0b01111111);
    println!("Weight: {}", mii_bytes.next().unwrap() & 0b01111111);

    let mut hex_str = String::new();
    for _ in 0..4 {
        hex_str += &format!("{:>02X}", mii_bytes.next().unwrap());
    }
    println!("Mii ID: {hex_str}");

    let mut hex_str = String::new();
    for _ in 0..4 {
        hex_str += &format!("{:>02X}", mii_bytes.next().unwrap());
    }
    println!("Console ID: {hex_str}");

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Face Shape: {}", byte1 >> 5);
    println!("Skin Tone: {}", (byte1 >> 2) & 0b00000111);
    println!(
        "Face Features: {}",
        ((byte1 << 2) | (byte2 >> 6)) & 0b00001111
    );
    println!("Can Mingle? {}", (byte2 & 0b00000100) > 0);
    println!("Source Type: {}", byte2 & 0b00000011);

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Hair Type: {}", byte1 >> 1);
    println!("Hair Color: {}", ((byte1 << 2) | (byte2 >> 6)) & 0b00000111);
    println!("Hair Flipped? {}", (byte2 & 0b00100000) > 0);

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Eyebrow Type: {}", byte1 >> 3);
    println!(
        "Eyebrow Rotation: {}",
        ((byte1 << 2) | (byte2 >> 6)) & 0b00001111
    );

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Eyebrow Color: {}", byte1 >> 5);
    println!("Eyebrow Size: {}", (byte1 >> 1) & 0b00001111);
    println!(
        "Eyebrow Vertical: {}",
        ((byte1 << 4) | (byte2 >> 4)) & 0b00011111
    );
    println!("Eyebrow Horizontal: {}", byte2 & 0b00001111);
    println!("Eye Type: {}", mii_bytes.next().unwrap() >> 2);

    let byte1 = mii_bytes.next().unwrap();

    println!("Eye Rotation: {}", byte1 >> 5);
    println!("Eye Vertical: {}", byte1 & 0b00001111);

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Eye Color: {}", byte1 >> 5);
    println!("Eye Size: {}", (byte1 >> 1) & 0b00000111);
    println!(
        "Eye Horizontal: {}",
        ((byte1 << 3) | (byte2 >> 5)) & 0b00001111
    );

    let byte1 = mii_bytes.next().unwrap();

    println!("Nose Type: {}", byte1 >> 4);
    println!("Nose Size: {}", byte1 & 0b00001111);
    println!("Nose Vertical: {}", mii_bytes.next().unwrap() >> 3);

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Mouth Type: {}", (byte1 >> 3));
    println!("Mouth Color: {}", ((byte1 >> 1) & 0b00000011));
    println!(
        "Mouth Size: {}",
        (((byte1 << 3) | (byte2 >> 5)) & 0b00001111)
    );
    println!("Mouth Vertical: {}", (byte2 & 0b00011111));

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Glasses Type: {}", byte1 >> 4);
    println!("Glasses Color: {}", (byte1 >> 1) & 0b00000111);
    println!(
        "Glasses Size: {}",
        ((byte1 << 3) | (byte2 >> 5)) & 0b00001111
    );
    println!("Glasses Vertical: {}", byte2 & 0b00011111);

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Facial Hair Mustache: {}", byte1 >> 6);
    println!("Facial Hair Beard: {}", (byte1 >> 4) & 0b00000011);
    println!("Facial Hair Color: {}", (byte1 >> 1) & 0b00000111);
    println!(
        "Facial Hair Size: {}",
        (((byte1 << 3) | (byte2 >> 5)) & 0b00001111)
    );
    println!("Facial Hair Vertical: {}", (byte2 & 0b00011111));

    let byte1 = mii_bytes.next().unwrap();
    let byte2 = mii_bytes.next().unwrap();

    println!("Mole Type: {}", (byte1 >> 7) > 0);
    println!("Mole Size: {}", ((byte1 >> 3) & 0b00001111));
    println!(
        "Mole Vertical: {}",
        (((byte1 << 2) | (byte2 >> 6)) & 0b00011111)
    );
    println!("Mole Horizontal: {}", ((byte2 >> 1) & 0b00011111));

    let mut str_bytes = vec![];
    str_bytes.reserve_exact(20);
    let mut last_was_null = false;
    let mut pop_out = 19;
    for index in 0..20 {
        let byte = *mii_bytes.next().unwrap();
        if pop_out != 0 {
            pop_out -= 1;
        }
        if last_was_null && byte == 0 {
            str_bytes.pop();
            break;
        }
        last_was_null = byte == 0;
        if index % 2 == 0 {
            str_bytes.push((byte as u16) << 8);
        } else {
            let last_byte = str_bytes.pop().unwrap();
            str_bytes.push(last_byte | (byte as u16));
        }
    }
    mii_bytes.nth(pop_out);
    println!(
        "Creator Name: \"{}\"",
        String::from_utf16(&str_bytes).unwrap()
    );
}
