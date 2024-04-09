use scraper::{Html, Selector};
use std::ffi::CStr;
use std::fs::{self, File};
use std::io::Write;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use url::Url;
use std::fmt;

// エラーの種類を表す列挙型
#[derive(Debug)]
pub enum WebsiteDownloaderError {
    ReqwestError(reqwest::Error),            // Reqwestライブラリのエラー
    IoError(std::io::Error),                  // 入出力関連のエラー
    UrlParseError(url::ParseError),          // URLパースエラー
    SelectorParseError(String),              // セレクターパースエラー
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

// ウェブサイトを保存
fn save_website(url: &str, directory: &str) -> Result<(), WebsiteDownloaderError> {
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; rv:124.0) Gecko/20100101 Firefox/124.0";
    let client = reqwest::blocking::Client::builder()
        .user_agent(user_agent)
        .https_only(false)
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(WebsiteDownloaderError::ReqwestError)?;

    let res = client.get(url).send().map_err(WebsiteDownloaderError::ReqwestError)?;
    let mut body = res.text().map_err(WebsiteDownloaderError::ReqwestError)?;

    fs::create_dir_all(directory).map_err(WebsiteDownloaderError::IoError)?;
    fs::create_dir_all(Path::new(directory).join("videos")).map_err(WebsiteDownloaderError::IoError)?;
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
    let video_selector = "video[src], source[src]";
    let parsed_video_selector = Selector::parse(video_selector).map_err(|e| WebsiteDownloaderError::SelectorParseError(format!("{:?}", e)))?;
    let video_elements = soup.select(&parsed_video_selector);
    for element in video_elements {
        if let Some(src) = element.value().attr("src") {
            let resolved_url = Url::parse(url).map_err(WebsiteDownloaderError::UrlParseError)?.join(src).map_err(WebsiteDownloaderError::UrlParseError)?;
            let content = client.get(resolved_url.as_str()).send().map_err(WebsiteDownloaderError::ReqwestError)?.bytes().map_err(WebsiteDownloaderError::ReqwestError)?;
            let file_name = resolved_url
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or("video.mp4");
            let file_path = Path::new(directory).join("videos").join(file_name);

            let mut file = File::create(&file_path).map_err(WebsiteDownloaderError::IoError)?;
            file.write_all(&content).map_err(WebsiteDownloaderError::IoError)?;

            let relative_path = format!("videos/{}", file_name);
            replacements.push((src.to_string(), relative_path));
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
