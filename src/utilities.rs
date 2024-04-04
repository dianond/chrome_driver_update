use regex::Regex;

pub fn get_version(str: &str) -> String {
    let re = Regex::new(r"\d+\.\d+\.\d+\.\d+").unwrap();
    for cap in re.captures_iter(str) {
        return cap[0].to_string();
    }
    return String::from("");
}

pub fn get_main_version(str: &str) -> String {
    let re = Regex::new(r"(1\d{2})\.\d+\.\d+\.\d+").unwrap();
    for cap in re.captures_iter(str) {
        return cap[1].to_string();
    }
    return String::from("");
}