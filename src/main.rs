use directories::BaseDirs;
use std::io::{Read, Write, stdin, stdout};

fn main() {
    let stdin = stdin();
    let mut stdout = stdout();
    let mut buf = String::new();

    let connection: sqlite::Connection;

    if let Some(base_dirs) = BaseDirs::new() {
        let diary_path = base_dirs.data_dir().join("simple-diary.db");
        connection = sqlite::open(diary_path).expect("unable to open database");
    } else {
        println!("Unable to determine user's home directory. Using same location as executable.");
        connection = sqlite::open("diary_entries.db").expect("unable to open database");
    }

    let query = "CREATE TABLE IF NOT EXISTS diary_entries (id INTEGER PRIMARY KEY AUTOINCREMENT, datetime TEXT, feeling_quant INTEGER, feeling_word TEXT, freeform_text TEXT)";
    connection.execute(query).expect("unable to execute query");

    let mut feeling_quant: u8 = 0;

    while feeling_quant == 0 {
        print!("Rate how you are feeling right now (1-100): ");
        stdout.flush().expect("unable to flush stdout");
        if let Ok(_) = stdin.read_line(&mut buf) {
            if let Ok(num) = buf.trim().parse::<u8>() {
                if num >= 1 && num <= 100 {
                    feeling_quant = num;
                } else {
                    println!("Invalid input. Please enter a number between 1 and 100.");
                }
            } else {
                println!("Invalid input. Please enter a number between 1 and 100.");
            }
        } else {
            println!("Invalid input. Please enter a number between 1 and 100.");
        }
        buf.clear();
    }


    let mut buf = String::new();

    let mut feeling_word = String::new();

    while feeling_word.is_empty() {
        print!("Using one word, how do you feel right now? ");
        stdout.flush().expect("unable to flush stdout");
        if let Ok(_) = stdin.read_line(&mut buf) {
            let input: Vec<&str> = buf.trim().split_whitespace().collect();

            if input.len() == 1 {
                if let Ok(_) = input[0].parse::<f64>() {
                    println!("Invalid input. Please enter a word.")
                } else {
                    feeling_word = input[0].to_string().to_lowercase();
                }
            } else {
                println!("Invalid input. Please enter a single word.");
            }
        }
        buf.clear();
    }

    let mut buf = String::new();
    let mut freeform_text = String::new();
    println!("Enter your thoughts below:\n");
    stdin.read_line(&mut buf).expect("unable to read line");

    while !buf.trim().is_empty() {
        freeform_text = freeform_text + &buf;
        buf.clear();

        stdin.read_line(&mut buf).expect("unable to read line");
        if buf.trim().is_empty() {
            freeform_text = freeform_text + &buf;
            buf.clear();
            stdin.read_line(&mut buf).expect("unable to read line");
        }
    }


    let query = "INSERT INTO diary_entries (datetime, feeling_quant, feeling_word, freeform_text) VALUES (datetime('now'), ?, ?, ?)";
    let mut statement = connection
        .prepare(query)
        .expect("unable to prepare statement");
    statement
        .bind((1, feeling_quant as i64))
        .expect("unable to bind parameter");
    statement
        .bind((2, &feeling_word[..]))
        .expect("unable to bind parameter");
    statement
        .bind((3, &freeform_text[..]))
        .expect("unable to bind parameter");

    statement.next().expect("unable to execute statement");

    println!("Entry Saved.")
}
