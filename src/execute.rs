use std::error::Error;
use reqwest::blocking::get;
use regex::Regex;
use crate::html_analyse::check_url;
use crate::run_shell::{run_cmd, run_powershell};

pub fn run() -> Result<String, Box<dyn Error>> {
    let chrome_driver_path = get_chrome_driver_path();
    if chrome_driver_path.is_empty() {
        return Err("Chrome driver path not found".into());
    }

    let chrome_version = get_chrome_version();
    let chrome_main_version = get_main_version(&chrome_version);
    if chrome_main_version.is_empty() {
        return Err("Chrome version not found".into());
    }

    let chrome_driver_version = get_chrome_driver_version();
    let chrome_driver_main_version = get_main_version(&chrome_driver_version);

    let mut url = String::new();
    if chrome_main_version == chrome_driver_main_version {
        url = get_chrome_driver_url_str(&chrome_main_version);
        if !url.is_empty() {
            let remote_version = get_version(&url);
            if remote_version == chrome_driver_version {
                return Ok(String::from("Chrome and Chrome driver version match"));
            }
        } else {
            return Err("Update not executed".into());
        }
    }

    println!("Chrome and Chrome driver version mismatch");

    if url.is_empty() {
        url = get_chrome_driver_url(&chrome_main_version)?;
    }

    println!("Chrome driver download url: {}", url);
    download_chrome_driver(&url)?;
    println!("Chrome driver download finish");

    unzip_chrome_driver()?;
    close_chrome_driver();
    println!("Chrome driver path: {}", chrome_driver_path);
    copy_chrome_driver(&chrome_driver_path)?;
    del_temp_file()?;
    del_temp_path()?;
    Ok(String::from("Finish"))
}

fn get_chrome_version() -> String {
    let command = "(Get-Item 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe').VersionInfo.ProductVersion";
    run_powershell(command).unwrap_or_else(|| String::from(""))
}

fn get_chrome_driver_version() -> String {
    let command = "chromedriver --version";
    let str = run_powershell(command).unwrap_or_else(|| String::from(""));
    get_version(&str)
}

fn get_chrome_driver_path() -> String {
    let command = "where chromedriver";
    run_cmd(command).unwrap_or_else(|| String::from(""))
}

fn get_version(str: &str) -> String {
    let re = Regex::new(r"\d+\.\d+\.\d+\.\d+").unwrap();
    for cap in re.captures_iter(str) {
        return cap[0].to_string();
    }
    return String::from("");
}

fn get_main_version(str: &str) -> String {
    let re = Regex::new(r"(1\d{2})\.\d+\.\d+\.\d+").unwrap();
    for cap in re.captures_iter(str) {
        return cap[1].to_string();
    }
    return String::from("");
}

fn get_chrome_driver_url(main_version: &str) -> Result<String, Box<dyn Error>> {
    let response = get("https://googlechromelabs.github.io/chrome-for-testing/")?;

    if response.status().is_success() {
        let body = response.text()?;
        let str = format!(
            r"https://storage.googleapis.com/chrome-for-testing-public/{}\.\d+\.\d+\.\d+/win64/chromedriver-win64.zip",
            main_version
        );
        let re = Regex::new(&str).unwrap();
        for cap in re.captures_iter(&body) {
            let url = cap[0].to_string();
            if check_url(&body, &url) {
                return Ok(url);
            }
        }
    }

    Err("get url failed".into())
}

fn get_chrome_driver_url_str(main_version: &str) -> String {
    get_chrome_driver_url(main_version).unwrap_or_else(|| String::from(""))
}

fn download_chrome_driver(url: &str) -> Result<(), Box<dyn Error>> {
    let response = get(url)?;

    if response.status().is_success() {
        let body = response.bytes()?;
        std::fs::write("chromedriver.zip", &body)?;
        return Ok(());
    }

    Err("download file failed".into())
}

fn unzip_chrome_driver() -> Result<(), Box<dyn Error>> {
    let command = "Expand-Archive -Path chromedriver.zip -DestinationPath .";
    match run_powershell(command) {
        Some(_) => {
            println!("Chrome driver unzip success");
            Ok(())
        }
        None => Err("Chrome driver unzip failed".into()),
    }
}

fn close_chrome_driver() {
    let command = "taskkill /F /IM chromedriver.exe";
    run_cmd(command);
}

fn copy_chrome_driver(chrome_driver_path: &str) -> Result<(), Box<dyn Error>> {
    let command = format!(
        "Copy-Item -Path 'chromedriver-win64\\chromedriver.exe' -Destination '{}'",
        chrome_driver_path
    );
    match run_powershell(&command) {
        Some(_) => {
            println!("Chrome driver copy success");
            Ok(())
        }
        None => Err("Chrome driver copy failed".into()),
    }
}

fn del_temp_file() -> Result<(), Box<dyn Error>> {
    let command = "Remove-Item -Path chromedriver.zip";
    match run_powershell(command) {
        Some(_) => {
            println!("Chrome driver temp file delete success");
            Ok(())
        }
        None => Err("Chrome driver temp file delete failed".into()),
    }
}

fn del_temp_path() -> Result<(), Box<dyn Error>> {
    let command = "Remove-Item -Path chromedriver-win64 -Recurse";
    match run_powershell(command) {
        Some(_) => {
            println!("Chrome driver temp path delete success");
            Ok(())
        }
        None => Err("Chrome driver temp path delete failed".into()),
    }
}
