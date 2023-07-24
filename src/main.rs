use std::ffi::OsStr;
use std::io::{Write};
use std::path::PathBuf;
use std::time::Instant;

// Base to div file size in bytes for
// If we div bytes on 1e6, we will receive file size in megabytes
const FILE_SIZE_BASE: f64 = 1e6;

// This function needed to make getting user input more convenient
// query parameter is a text we printing to console when asking user for input
fn get_input(query: &str) -> std::io::Result<String> {
    print!("{}", query);
    std::io::stdout().flush()?;

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    // returning String without redundant spaces and transitions to a new line
    Ok(buffer.trim().to_owned())
}

// This function gets all needed for file search data from user
// Function also handles possible invalid input
fn get_search_data() -> Option<(String, String, Vec<String>)> {
    let search_path = match get_input("Enter path to dir to search for file: ") {
        Ok(path) => path,
        Err(err) => {
            println!("Error getting user input, try again: {}\n", err);
            return None;
        }
    };
    let search_name = match get_input("Enter a file name to search (without extension): ") {
        Ok(name) => name,
        Err(err) => {
            println!("Error getting user input, try again: {}\n", err);
            return None;
        }
    };
    let extensions = match get_input("Enter file extensions separated by space: ") {
        Ok(extensions) => get_extensions(extensions),
        Err(err) => {
            println!("Error getting user input, try again: {}\n", err);
            return None;
        }
    };

    // Handling possible invalid input
    if search_path.is_empty() || (search_name.is_empty() && extensions.is_empty()) {
        println!("You must enter the path to search and either a filename or extensions");
        return None;
    }

    Some((search_path.to_lowercase(), search_name.to_lowercase(), extensions))
}

// This function splits extensions list string by spaces and returns
// vector of strings with file extensions to search later
fn get_extensions(extensions_string: String) -> Vec<String> {
    extensions_string.split_whitespace().map(|word| word.to_lowercase()).collect()
}

// This function is needed to do converting OsStr to String more convenient
// Also this function puts given text to lowercase
fn os_str_to_str(os_str: Option<&OsStr>) -> String {
    os_str
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase()
}

// This function utilizes processes when filesystem object is found
// Function increments objects count and calls method that prints found object info
fn object_was_found(path: &PathBuf, now: &Instant, results_count: &mut i32) {
    *results_count += 1;
    print_path_info(&path, now);
}

// This function prints an information about given path (absolute path to file, file size)
// Also function prints time elapsed to find this file
fn print_path_info(path: &PathBuf, now: &Instant) {
    print!(
        "{} - Found in {} seconds",
        path.display(),
        now.elapsed().as_secs_f64()
    );

    // Fetching file size
    match std::fs::metadata(path) {
        Ok(metadata) => {
            print!(" - {} MB\n", metadata.len() as f64 / FILE_SIZE_BASE)
        }
        Err(_) => println!()
    }
}

// This function utilizes file searching functionality
// Function takes a path to dir where to search, filename, file extensions to search for
// and also additional counters (time and found objects counter)
// Function searches for needed files recursively going trough every directory
// in given path
// Function can search: only for filename (without extension), only for extension (or
// several extensions), for both filename and extension(s)
fn search_files(search_dir: &str, filename: &str, extensions: &Vec<String>,
                now: &Instant, results_count: &mut i32) {
    let no_extensions = extensions.is_empty();
    let empty_filename = filename.is_empty();

    // Fetching files in current dir
    let files = match std::fs::read_dir(search_dir) {
        Ok(files) => files,
        Err(_) => return // Error, skip this dir
    };

    for entry in files {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_name = os_str_to_str(path.file_stem());
            let file_extension = os_str_to_str(path.extension());

            if path.is_dir() {
                if no_extensions && file_name.contains(filename) {
                    // Dir matches by filename
                    object_was_found(&path, now, results_count);
                }

                // Going trough this dir recursively
                search_files(path.to_str().unwrap_or_default(), filename, extensions, now, results_count);
            } else if empty_filename && extensions.contains(&file_extension) {
                object_was_found(&path, now, results_count);
            } else if path.is_file() && file_name.contains(filename) {
                if (!no_extensions && extensions.contains(&file_extension)) || no_extensions {
                    object_was_found(&path, now, results_count);
                }
            }
        }
    }
}

// Program entry point
fn main() {
    // Main console program loop
    loop {
        // Receiving needed for file search data
        let (search_path, search_name, extensions) = match get_search_data() {
            None => continue,
            Some(data) => data
        };

        println!();

        // Program counters
        let now = Instant::now(); // Time counter
        let mut results_count = 0; // Found objects counter

        // Executing file search
        search_files(
            search_path.as_str(),
            search_name.as_str(),
            &extensions,
            &now,
            &mut results_count,
        );

        // Search total results (time elapsed and found results amount)
        println!(
            "\nTotal time: {} seconds\n{} results found\n",
            now.elapsed().as_secs_f64(),
            results_count
        );
    }
}
