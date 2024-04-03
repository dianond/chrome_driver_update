mod html_analyse;
mod run_shell;
mod execute;

use std::error::Error;
use execute::run;

fn main() {
    match run() {
        Ok(ok) => {
            println!("Success: {}", ok);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
