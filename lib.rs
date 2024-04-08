use scraper::{Html, Selector};
use std::ffi::CStr;
use std::fs::{self, File};
use std::io::Write;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use url::Url;
use std::fmt;

#[derive(Debug)]
pub enum WebsiteDownloaderError {
    ReqwestError(reqwest::Error),
    IoError(std::io::Error),
    UrlParseError(url::ParseError),
    SelectorParseError(String),
}

impl fmt::Display for WebsiteDownloaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WebsiteDownloaderError::ReqwestError(ref err) => write!(f, "Request error: {}", err),
            WebsiteDownloaderError::IoError(ref err) => write!(f, "IO error: {}", err),
            WebsiteDownloaderError::UrlParseError(ref err) => write!(f, "URL parse error: {}", err),
            WebsiteDownloaderError::SelectorParseError(ref err) => write!(f, "Selector parse error: {}", err),
        }
    }
}

impl std::error::Error for WebsiteDownloaderError {}
//savedwebsute
fn save_website(url: &str, directory: &str) -> Result<(), WebsiteDownloaderError> {
    let client = reqwest::blocking::Client::new();
    let res = client.get(url).send().map_err(WebsiteDownloaderError::ReqwestError)?;
    let mut body = res.text().map_err(WebsiteDownloaderError::ReqwestError)?;

    fs::create_dir_all(directory).map_err(WebsiteDownloaderError::IoError)?;
    for sub_dir in ["css", "js", "img"].iter() {
        fs::create_dir_all(Path::new(directory).join(sub_dir)).map_err(WebsiteDownloaderError::IoError)?;
    }

    let soup = Html::parse_document(&body);
    let mut replacements = Vec::new();

    for selector in ["link[rel='stylesheet']", "script[src]", "img[src]"].iter() {
        let parsed_selector = Selector::parse(selector).map_err(|e| WebsiteDownloaderError::SelectorParseError(format!("{:?}", e)))?;
        let elements = soup.select(&parsed_selector);
        for element in elements {
            let attr = if element.value().name() == "link" { "href" } else { "src" };
            if let Some(href) = element.value().attr(attr) {
                let resolved_url = Url::parse(url).map_err(WebsiteDownloaderError::UrlParseError)?.join(href).map_err(WebsiteDownloaderError::UrlParseError)?;
                let content = client.get(resolved_url.as_str()).send().map_err(WebsiteDownloaderError::ReqwestError)?.bytes().map_err(WebsiteDownloaderError::ReqwestError)?;
                let file_name = resolved_url
                    .path_segments()
                    .and_then(|segments| segments.last())
                    .unwrap_or("index.html");
                let sub_directory = match element.value().name() {
                    "link" => "css",
                    "script" => "js",
                    _ => "img",
                };
                let file_path = Path::new(directory).join(sub_directory).join(file_name);

                let mut file = File::create(&file_path).map_err(WebsiteDownloaderError::IoError)?;
                file.write_all(&content).map_err(WebsiteDownloaderError::IoError)?;

                let relative_path = format!("{}/{}", sub_directory, file_name);
                replacements.push((href.to_string(), relative_path));
            }
        }
    }

    for (original, replacement) in replacements {
        body = body.replace(&original, &replacement);
    }

    fs::write(Path::new(directory).join("index.html"), body).map_err(WebsiteDownloaderError::IoError)?;

    Ok(())
}

#[no_mangle]
pub extern "C" fn save_website_extern(url: *const c_char, directory: *const c_char) -> c_int {
    let url = unsafe {
        assert!(!url.is_null());
        match CStr::from_ptr(url).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };
    let directory = unsafe {
        assert!(!directory.is_null());
        match CStr::from_ptr(directory).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    match save_website(url, directory) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}