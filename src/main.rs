use regex::Regex;
use reqwest::blocking::get;
use std::error::Error;
use std::process::Command;

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

fn run() -> Result<String, Box<dyn Error>> {
    let chrome_driver_path = get_chrome_driver_path();
    let chrome_driver_path = chrome_driver_path.trim();
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

    if chrome_main_version == chrome_driver_main_version {
        return Ok(String::from("Chrome and Chrome driver version match"));
    } else {
        println!("Chrome and Chrome driver version mismatch");
        let url = get_chrome_driver_url(&chrome_main_version)?;
        if url.is_empty() {
            return Err("Chrome driver download url not found".into());
        }

        println!("Chrome driver download url: {}", url);
        download_chrome_driver(url.as_str())?;
        println!("download Chrome driver finish");

        unzip_chrome_driver()?;
        close_chrome_driver();
        println!("Chrome driver path: {}", chrome_driver_path);
        copy_chrome_driver(&chrome_driver_path)?;
        del_temp_file()?;
        del_temp_path()?;
    }

    Ok(String::from("Finish"))
}

fn run_powershell(command: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args(["-Command", command])
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).expect("Not UTF-8");
        let stdout = stdout.trim().to_string();
        return Some(stdout);
    }
    return None;
}

fn run_cmd(command: &str) -> Option<String> {
    let output = Command::new("cmd")
        .args(["/C", command])
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).expect("Not UTF-8");
        let stdout = stdout.trim().to_string();
        return Some(stdout);
    }
    return None;
}

fn get_chrome_version() -> String {
    let command = "(Get-Item 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe').VersionInfo.ProductVersion";
    match run_powershell(command) {
        Some(s) => s,
        None => String::from(""),
    }
}

fn get_chrome_driver_version() -> String {
    let command = "chromedriver --version";
    let str = match run_powershell(command) {
        Some(s) => s,
        None => String::from(""),
    };
    let re = Regex::new(r"\d+\.\d+\.\d+\.\d+").unwrap();
    for cap in re.captures_iter(&str) {
        return cap[0].to_string();
    }
    return String::from("");
}

fn get_chrome_driver_path() -> String {
    let command = "where chromedriver";
    match run_cmd(command) {
        Some(s) => s,
        None => String::from(""),
    }
}

fn get_main_version(str: &str) -> String {
    let re = Regex::new(r"(1\d{2})\.\d+\.\d+\.\d+").unwrap();
    for cap in re.captures_iter(str) {
        return cap[1].to_string();
    }
    return String::from("");
}

fn get_chrome_driver_url(version: &str) -> Result<String, Box<dyn Error>> {
    let response = get("https://googlechromelabs.github.io/chrome-for-testing/")?;

    if response.status().is_success() {
        let body = response.text()?;
        let str = format!(
            r"https://storage.googleapis.com/chrome-for-testing-public/{}\.\d+\.\d+\.\d+/win64/chromedriver-win64.zip",
            version
        );
        let re = Regex::new(&str).unwrap();
        for cap in re.captures_iter(&body) {
            return Ok(cap[0].to_string());
        }
    }

    Err("get url failed".into())
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
        Some(_s) => {
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
        Some(_s) => {
            println!("Chrome driver copy success");
            Ok(())
        }
        None => Err("Chrome driver copy failed".into()),
    }
}

fn del_temp_file() -> Result<(), Box<dyn Error>> {
    let command = "Remove-Item -Path chromedriver.zip";
    match run_powershell(command) {
        Some(_s) => {
            println!("Chrome driver temp file delete success");
            Ok(())
        }
        None => Err("Chrome driver temp file delete failed".into()),
    }
}

fn del_temp_path() -> Result<(), Box<dyn Error>> {
    let command = "Remove-Item -Path chromedriver-win64 -Recurse";
    match run_powershell(command) {
        Some(_s) => {
            println!("Chrome driver temp path delete success");
            Ok(())
        }
        None => Err("Chrome driver temp path delete failed".into()),
    }
}
