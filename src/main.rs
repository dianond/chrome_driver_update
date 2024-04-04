mod html_analyse;
mod run_shell;
mod execute;
mod browser;
mod utilities;

use execute::run;
use crate::browser::BrowserEnum;

fn main() {
    match run(BrowserEnum::Edge) {
        Ok(ok) => {
            println!("Success: {}", ok);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    match run(BrowserEnum::Chrome) {
        Ok(ok) => {
            println!("Success: {}", ok);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
