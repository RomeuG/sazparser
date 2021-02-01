use std::fs;
use std::io::BufReader;

use regex::Regex;

use std::rc::Rc;

use std::error;
use std::fmt;

mod constants {
    pub const MAGIC_SAZ: &[u8; 4] = b"\x50\x4B\x03\x04";
    pub const MAGIC_SAZ_EMPTY: &[u8; 4] = b"\x50\x4B\x05\x06";
    pub const MAGIC_SAZ_SPANNED: &[u8; 4] = b"\x50\x4B\x07\x08";
}

type Result<T> = std::result::Result<T, SazError>;

#[derive(Debug)]
/// Library Error
pub enum SazError {
    /// SAZ/ZIP file is empty
    Empty,
    /// SAZ/ZIP file is spanned
    Spanned,
    /// SAZ/ZIP file is invalid
    Invalid,
    /// Failure in reading file
    Error,
}

impl fmt::Display for SazError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SazError::Empty => write!(f, "SAZ file is empty"),
            SazError::Spanned => write!(f, "SAZ file is spanned"),
            SazError::Invalid => write!(f, "SAZ file is invalid"),
            SazError::Error => write!(f, "Error reading file"),
        }
    }
}

impl error::Error for SazError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            SazError::Empty => None,
            SazError::Spanned => None,
            SazError::Invalid => None,
            SazError::Error => None,
        }
    }
}

#[derive(Clone, Debug)]
struct SazFile {
    path: String,
    size: u64,
    contents: Rc<String>,
}

impl SazFile {
    fn new(path: &String, size: u64, contents: Rc<String>) -> SazFile {
        SazFile {
            path: path.to_string(),
            size: size,
            contents: contents.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// This struct represent a single SAZ session
pub struct SazSession {
    /// Identifier
    pub index: u32,
    /// Http Status Code
    pub result: u32,
    /// Request URL
    pub url: String,
    /// HTTP Body length in bytes
    pub body: u32,
    /// File with Request information
    pub file_request: String,
    /// File with Response information
    pub file_response: String,
    /// Contents of Request file
    pub file_request_contents: Rc<String>,
    /// Contents of Response file
    pub file_response_contents: Rc<String>,
}

impl SazSession {
    fn new(
        idx: u32,
        httpres: u32,
        httpurl: &str,
        httpbody: u32,
        frequest: &String,
        fresponse: &String,
        frequest_contents: &Rc<String>,
        fresponse_contents: &Rc<String>,
    ) -> SazSession {
        SazSession {
            index: idx,
            result: httpres,
            url: httpurl.to_string(),
            body: httpbody,
            file_request: frequest.clone(),
            file_response: fresponse.clone(),
            file_request_contents: frequest_contents.clone(),
            file_response_contents: fresponse_contents.clone(),
        }
    }
}

fn check_file_validity(filename: &str) -> Result<()> {
    let data = fs::read(filename);

    if let Ok(d) = data {
        let slice = &d[..4];
        if slice == constants::MAGIC_SAZ {
            return Ok(());
        } else if slice == constants::MAGIC_SAZ_EMPTY {
            return Err(SazError::Empty);
        } else if slice == constants::MAGIC_SAZ_SPANNED {
            return Err(SazError::Spanned);
        } else {
            return Err(SazError::Invalid);
        }
    } else {
        return Err(SazError::Error);
    }
}

fn zip_contents(filename: &str) -> Result<Vec<SazFile>> {
    check_file_validity(filename)?;

    let mut raw_folder: bool = false;
    let mut list: Vec<SazFile> = vec![];

    let file = fs::File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader).unwrap();

    for i in 0..archive.len() {
        let mut zipped_file = archive.by_index(i).unwrap();

        let outpath = match zipped_file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        if !zipped_file.name().ends_with('/') {
            let file_path = outpath.to_str().unwrap().to_string();
            let file_size = zipped_file.size();

            let mut writer: Vec<u8> = vec![];
            let _ = std::io::copy(&mut zipped_file, &mut writer);
            let file_contents = unsafe { std::str::from_utf8_unchecked(&writer).to_string() };

            let zippedfile = SazFile::new(&file_path, file_size, Rc::new(file_contents));
            list.push(zippedfile);
        } else {
            if zipped_file.name() == "raw/" {
                raw_folder = true;
            }
        }
    }

    if !raw_folder {
        return Err(SazError::Invalid);
    }

    Ok(list)
}

fn get_file_from_list<'a>(list: &'a [SazFile], filename: &str) -> &'a SazFile {
    let result = list.iter().find(|&f| f.path == filename).unwrap();
    result
}

fn get_sessions_total(list: &[SazFile]) -> (u32, usize) {
    let mut leading_zeroes: usize = 0;
    let mut sessions_total: u32 = 0;

    for zipped_file in list {
        let mut splitted = zipped_file.path.split('_');
        let splitted2 = splitted.nth(0);
        match splitted2 {
            Some(inner) => {
                let number = inner.split('/').nth(1);
                match number {
                    Some(inner2) => {
                        leading_zeroes = inner2.len();
                        let parsed = inner2.parse::<u32>().unwrap();
                        if sessions_total < parsed {
                            sessions_total = parsed;
                        }
                    }
                    None => continue,
                }
            }
            None => continue,
        }
    }

    (sessions_total, leading_zeroes)
}

fn regex_get_url(contents: &str) -> &str {
    let re = Regex::new(r"(GET|HEAD|POST|PUT|DELETE|CONNECT|OPTIONS|TRACE) (.*) HTTP/1.1").unwrap();
    let capture = re.captures(contents).unwrap();
    let value = capture.get(2).unwrap().as_str();

    value
}

fn regex_get_http_status(contents: &str) -> u32 {
    let re = Regex::new(r"HTTP.*([0-9][0-9][0-9])").unwrap();
    let capture = re.captures(contents).unwrap();
    let value = capture.get(1).unwrap().as_str().parse::<u32>().unwrap();

    value
}

fn regex_get_content_length(contents: &str) -> u32 {
    let mut value: u32 = 0;

    let re = Regex::new(r"Content-Length:\s(\d+)").unwrap();
    let capture = re.captures(contents);

    match capture {
        Some(captured) => {
            value = captured.get(1).unwrap().as_str().parse::<u32>().unwrap();
        }
        None => return value,
    }

    value
}

///
/// Parses given file.
/// Returns Result<Vec<SazSession>>
///
/// # Arguments
///
/// * `fname` - File name that represents the SAZ file
///
/// # Errors
///
/// Errors out if not possible to read file.
///
/// # Example
///
/// ``` rust
/// use std::env;
///
/// use sazparser;
///
/// fn main() {
///     let args: Vec<String> = env::args().collect();
///
///     // args[1] will be the file to parse
///     let saz = sazparser::parse(&*args[1]);
///
///     match saz {
///         Ok(v) => {
///             // use parsed information
///             println!("{:?}", v);
///         },
///         Err(e) => {
///             panic!("{}", e);
///         },
///     }
/// }
///```
///
pub fn parse(fname: &str) -> Result<Vec<SazSession>> {
    let mut entries: Vec<SazSession> = vec![];

    let list = zip_contents(&fname)?;
    let (total_sessions, leading_zeroes) = get_sessions_total(&list);

    for n in 1..=total_sessions {
        let request_file_name = format!("raw/{:0fill$}_c.txt", n, fill = leading_zeroes);
        let response_file_name = format!("raw/{:0fill$}_s.txt", n, fill = leading_zeroes);

        let request_file = get_file_from_list(&list, &*request_file_name);
        let response_file = get_file_from_list(&list, &*response_file_name);

        let url = regex_get_url(&*request_file.contents);
        let httpstatus = regex_get_http_status(&*response_file.contents);
        let contentlength = regex_get_content_length(&*response_file.contents);

        let entry = SazSession::new(
            n,
            httpstatus,
            url,
            contentlength,
            &request_file_name,
            &response_file_name,
            &request_file.contents,
            &response_file.contents,
        );

        entries.push(entry);
    }

    Ok(entries)
}
