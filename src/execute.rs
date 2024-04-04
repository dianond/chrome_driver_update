use std::error::Error;
use crate::browser::{Browser, BrowserEnum};
use crate::utilities::get_version;

pub fn run(browser_enum: BrowserEnum) -> Result<String, Box<dyn Error>> {
    let browser = Browser::new(&browser_enum);
    if browser.main_version.is_empty() {
        let err_str = format!("{} version not found", browser.browser);
        return Err(err_str.into());
    }
    let browser_driver = browser.get_browser_driver();
    if browser_driver.driver_path.is_empty() {
        let err_str = format!("{} driver path not found", browser_driver.name);
        return Err(err_str.into());
    }

    let mut url = String::new();
    if browser.main_version == browser_driver.main_version {
        url = browser_driver.get_driver_url(&browser);
        if !url.is_empty() {
            let remote_version = get_version(&url);
            if remote_version == browser_driver.version {
                return Ok(format!("{} and {} version match", browser.browser, browser_driver.name));
            }
        } else {
            return Err("Update not executed".into());
        }
    }

    println!("{} and {} version mismatch", browser.browser, browser_driver.name);

    if url.is_empty() {
        url = browser_driver.get_driver_url(&browser);
    }

    println!("{} download url: {}", browser_driver.name, url);
    browser_driver.download_driver(&url)?;
    println!("{} download finish", browser_driver.name);

    browser_driver.unzip_driver()?;
    browser_driver.close_driver();
    println!("{} path: {}", browser_driver.name, browser_driver.driver_path);
    browser_driver.copy_driver(&browser_driver.driver_path)?;
    browser_driver.del_temp_file()?;
    browser_driver.del_temp_path()?;
    Ok(String::from("Finish"))
}
