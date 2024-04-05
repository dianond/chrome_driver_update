use std::error::Error;
use std::fmt::{Display, Formatter};
use regex::Regex;
use reqwest::blocking::get;
use crate::html_analyse::check_url;
use crate::run_shell::{run_cmd, run_powershell};
use crate::utilities::{get_main_version, get_version};

#[derive(Clone)]
pub(crate) enum BrowserEnum {
    Chrome,
    Edge,
}

impl Display for BrowserEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserEnum::Chrome => write!(f, "Chrome"),
            BrowserEnum::Edge => write!(f, "Edge"),
        }
    }
}

pub(crate) struct Browser {
    pub(crate) browser: BrowserEnum,
    pub(crate) version: String,
    pub(crate) main_version: String,
}

impl Browser {
    pub(crate) fn new(browser_enum: &BrowserEnum) -> Self {
        let browser = browser_enum.clone();

        let version = match browser_enum {
            BrowserEnum::Chrome => {
                let command = "(Get-Item 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe').VersionInfo.ProductVersion";
                run_powershell(command).unwrap_or_else(|| String::from(""))
            }
            BrowserEnum::Edge => {
                let command = "(Get-Item 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe').VersionInfo.ProductVersion";
                run_powershell(command).unwrap_or_else(|| String::from(""))
            }
        };

        let main_version = get_main_version(&version);

        Self { browser, version, main_version }
    }

    pub(crate) fn get_browser_driver(&self) -> BrowserDriver {
        BrowserDriver::new(&self.browser)
    }
}

pub(crate) struct BrowserDriver {
    pub(crate) browser: BrowserEnum,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) main_version: String,
    pub(crate) driver_path: String,
}

impl BrowserDriver {
    pub(crate) fn new(browser_enum: &BrowserEnum) -> Self {
        let browser = browser_enum.clone();

        let name = match browser_enum {
            BrowserEnum::Chrome => String::from("chromedriver"),
            BrowserEnum::Edge => String::from("msedgedriver"),
        };

        let command = format!("{} --version", &name);
        let str = run_powershell(&command).unwrap_or_else(|| String::from(""));
        let (version, main_version) = get_version(&str);

        let driver_path = get_browser_driver_path(&name);

        Self { browser, name, version, main_version, driver_path }
    }

    pub(crate) fn get_driver_url(&self, browser: &Browser) -> String {
        match self.browser {
            BrowserEnum::Chrome => {
                get_chrome_driver_url(&browser.main_version).unwrap_or_else(|_| String::from(""))
            }
            BrowserEnum::Edge => {
                get_edge_driver_url(&browser.version).unwrap_or_else(|_| String::from(""))
            }
        }
    }

    pub(crate) fn download_driver(&self, url: &str) -> Result<(), Box<dyn Error>> {
        let response = get(url)?;

        if response.status().is_success() {
            let body = response.bytes()?;
            let path = format!("{}.zip", &self.name);
            std::fs::write(&path, &body)?;
            return Ok(());
        }

        Err("download file failed".into())
    }

    pub(crate) fn unzip_driver(&self) -> Result<(), Box<dyn Error>> {
        let command = format!(
            "Expand-Archive -Path '{}.zip' -Destination .",
            &self.name
        );
        match run_powershell(&command) {
            Some(_) => {
                println!("{} unzip success", &self.name);
                Ok(())
            }
            None => Err(format!("{} unzip failed", &self.name).into()),
        }
    }

    pub(crate) fn close_driver(&self) {
        let command = format!("taskkill /f /im {}.exe", &self.name);
        run_cmd(&command);
    }

    pub(crate) fn copy_driver(&self, driver_path: &str) -> Result<(), Box<dyn Error>> {
        let command = match self.browser {
            BrowserEnum::Chrome => {
                format!(
                    "Copy-Item -Path 'chromedriver-win64\\chromedriver.exe' -Destination '{}'",
                    driver_path
                )
            }
            BrowserEnum::Edge => {
                format!(
                    "Copy-Item -Path 'msedgedriver.exe' -Destination '{}'",
                    driver_path
                )
            }
        };
        match run_powershell(&command) {
            Some(_) => {
                println!("{} copy success", &self.name);
                Ok(())
            }
            None => Err(format!("{} copy failed", &self.name).into()),
        }
    }

    pub(crate) fn del_temp_file(&self) -> Result<(), Box<dyn Error>> {
        let command = format!(
            "Remove-Item -Path '{}.zip'",
            &self.name
        );
        match run_powershell(&command) {
            Some(_) => {
                println!("{} temp file delete success", &self.name);
                Ok(())
            }
            None => Err(format!("{} temp file delete failed", &self.name).into()),
        }
    }

    pub(crate) fn del_temp_path(&self) -> Result<(), Box<dyn Error>> {
        match self.browser {
            BrowserEnum::Chrome => {
                let command = "Remove-Item -Path chromedriver-win64 -Recurse";
                match run_powershell(command) {
                    Some(_) => {
                        println!("chromedriver temp path delete success");
                        Ok(())
                    }
                    None => Err("chromedriver temp path delete failed".into()),
                }
            }
            BrowserEnum::Edge => {
                let command = "Remove-Item -Path Driver_Notes -Recurse";
                if let None = run_powershell(command) {
                    return Err("msedgedriver temp path delete failed".into());
                }
                let command = "Remove-Item -Path msedgedriver.exe";
                match run_powershell(command) {
                    Some(_) => {
                        println!("msedgedriver temp path delete success");
                        Ok(())
                    }
                    None => Err("msedgedriver temp path delete failed".into()),
                }
            }
        }
    }
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

fn get_edge_driver_url(version: &str) -> Result<String, Box<dyn Error>> {
    let response = get("https://developer.microsoft.com/zh-cn/microsoft-edge/tools/webdriver/?form=MA13LH")?;

    if response.status().is_success() {
        let body = response.text()?;
        let str = format!(
            r"https://msedgedriver.azureedge.net/{}/edgedriver_win64.zip",
            version
        );
        let re = Regex::new(&str).unwrap();
        for cap in re.captures_iter(&body) {
            let url = cap[0].to_string();
            return Ok(url);
        }
    }

    Err("get url failed".into())
}

fn get_browser_driver_path(name: &str) -> String {
    let command = format!("where {}", name);
    run_cmd(&command).unwrap_or_else(|| String::from(""))
}