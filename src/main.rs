use config::Config;
use directories::BaseDirs;
use std::collections::HashMap;
use std::io::{Write, stdin, stdout};

const DEFAULT_DB_NAME: &'static str = "diary_entries.db";

fn main() {
    let stdin: std::io::Stdin = stdin();
    let mut stdout: std::io::Stdout = stdout();
    let mut buf: String = String::new();

    let connection: sqlite::Connection;

    let settings: HashMap<String, String> = get_settings();

    println!("{:?}", settings);

    let db_name = if let Some(db_name_setting) = settings.get("db_name") {
        db_name_setting.clone()
    } else {
        String::from(DEFAULT_DB_NAME)
    };

    connection = connect_to_database(&db_name[..]);

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

fn connect_to_database(diary_path: &str) -> sqlite::Connection {
    println!("Connecting to database at {}", diary_path);
    sqlite::open(diary_path).expect("unable to open database")
}

fn get_default_db_path() -> String {
    if let Some(base_dirs) = BaseDirs::new() {
        let path_buf = base_dirs.data_dir().join(DEFAULT_DB_NAME);
        if let Some(path_str) = path_buf.to_str() {
            String::from(path_str)
        } else {
            String::from(DEFAULT_DB_NAME)
        }
    } else {
        String::from(DEFAULT_DB_NAME)
    }
}

fn get_settings() -> HashMap<String, String> {
    let mut full_config_dir: String = String::from("");

    if let Some(base_dirs) = BaseDirs::new() {
        let mut config_dir = base_dirs.config_dir().to_path_buf();
        config_dir = config_dir.join("simple-diary");
        config_dir = config_dir.join("settings");

        if let Some(config_dir_str) = config_dir.to_str() {
            full_config_dir = String::from(config_dir_str);
        }
    }

    let mut local_settings_toml = config::File::with_name("settings.toml");
    local_settings_toml = local_settings_toml.required(false);

    let mut config_dir_toml = config::File::with_name(&full_config_dir[..]);
    config_dir_toml = config_dir_toml.required(false);

    let config_builder = Config::builder()
        .add_source(local_settings_toml)
        .add_source(config_dir_toml);

    let config_builder = config_builder
        .set_default("db_name", get_default_db_path())
        .expect("couldn't set default db path");

    let settings = config_builder.build().expect("Couldn't build settings");

    settings
        .try_deserialize::<HashMap<String, String>>()
        .expect("unable to deserialize settings")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlite::State;
    use std::fs::File;
    use std::fs::remove_file;
    use std::io::prelude;

    #[test]
    fn test_local_settings_file() {
        let mut local_settings_file =
            File::create("settings.toml").expect("unable to open settings-test.toml");
        local_settings_file
            .write_all(b"db_name = \"config-test.db\"")
            .expect("unable to write to settings-test.toml");

        let settings = get_settings();
        assert_eq!(settings.get("db_name").unwrap(), "config-test.db");

        remove_file("settings.toml").expect("unable to remove settings.toml");
    }

    #[test]
    fn test_db_creation_and_insertion() {
        let mut local_settings_file =
            File::create("settings.toml").expect("unable to open settings-test.toml");

        local_settings_file
            .write_all(b"db_name = \"db-test.db\"")
            .expect("unable to write to settings-test.toml");

        let settings = get_settings();

        let db_name = settings.get("db_name").unwrap();

        let connection = connect_to_database(db_name);

        let query = "CREATE TABLE IF NOT EXISTS diary_entries (id INTEGER PRIMARY KEY AUTOINCREMENT, datetime TEXT, feeling_quant INTEGER, feeling_word TEXT, freeform_text TEXT)";

        connection.execute(query).expect("unable to create table");

        let query = "INSERT INTO diary_entries (datetime, feeling_quant, feeling_word, freeform_text) VALUES ('2022-01-01 00:00:00', 5, 'happy', 'I am happy')";
        connection.execute(query).expect("unable to insert data");

        let query = "SELECT * FROM diary_entries WHERE id = 0";
        let mut stmt = connection
            .prepare(query)
            .expect("unable to prepare statement");

        while stmt.next().expect("Could not advance state.") == State::Row {
            let id: f64 = stmt.read(0).unwrap();
            let datetime: String = stmt.read(1).unwrap();
            let feeling_quant: f64 = stmt.read(2).unwrap();
            let feeling_word: String = stmt.read(3).unwrap();
            let freeform_text: String = stmt.read(4).unwrap();

            assert!(id == 1.0);
            assert!(datetime == "2022-01-01 00:00:00");
            assert!(feeling_quant == 5.0);
            assert!(feeling_word == "happy");
            assert!(freeform_text == "I am happy");
        }

        remove_file("config-test.db").expect("could not remove config-test.db");
    }
}
