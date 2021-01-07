pub mod fiddlerparser {

    use std::fs;
    use std::io::BufReader;

    use regex::Regex;

    use std::rc::Rc;

    mod constants {
	pub const MAGIC_ZIP: &[u8; 4] = b"\x50\x4B\x03\x04";
	pub const MAGIC_ZIP_EMPTY: &[u8; 4] = b"\x50\x4B\x03\x04";
	pub const MAGIC_ZIP_SPANNED: &[u8; 4] = b"\x50\x4B\x03\x04";
    }

    #[derive(Debug)]
    struct FiddlerError;
    impl std::fmt::Display for FiddlerError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "FiddlerError");
        }
    }
    impl std::error::Error for FiddlerError {}

    #[derive(Debug)]
    pub enum ZipValidity {
	Valid,
	Empty,
	Spanned,
	Invalid,
        Error
    }

    #[derive(Debug, Clone)]
    struct ZippedFile {
	path: String,
	size: u64,
	contents: Rc<String>
    }

    impl ZippedFile {
        fn new(path: &String, size: u64, contents: Rc<String>) -> ZippedFile {
            ZippedFile {
                path: path.to_string(),
                size: size,
                contents: contents.clone()
            }
        }
    }

    #[derive(Debug)]
    pub struct FiddlerEntry {
	index: u32,
	result: u32,
	url: String,
	body: u32,
	file_request: String,
	file_response: String,
	file_request_contents: Rc<String>,
	file_response_contents: Rc<String>
    }

    impl FiddlerEntry {
        fn new(
            idx: u32,
            httpres: u32,
            httpurl: &str,
            httpbody: u32,
            frequest: &String,
            fresponse: &String,
            frequest_contents: &Rc<String>,
            fresponse_contents: &Rc<String>
        ) -> FiddlerEntry {
            FiddlerEntry {
                index: idx,
	        result: httpres,
	        url: httpurl.to_string(),
	        body: httpbody,
	        file_request: frequest.clone(),
	        file_response: fresponse.clone(),
	        file_request_contents: frequest_contents.clone(),
	        file_response_contents: fresponse_contents.clone()
            }
        }
    }

    fn check_file_validity(filename: &str) -> Result<(), ZipValidity> {
	let data = fs::read(filename).unwrap();

	let slice = &data[..4];
	if slice == constants::MAGIC_ZIP {
	    return Ok(());
	} else if slice == constants::MAGIC_ZIP_EMPTY {
	    return Err(ZipValidity::Empty);
	} else if slice == constants::MAGIC_ZIP_SPANNED {
	    return Err(ZipValidity::Spanned);
	} else {
	    return Err(ZipValidity::Invalid);
	}
    }

    //fn zip_contents(filename: &str) -> zip::result::ZipResult<Vec<ZippedFile>>
    fn zip_contents(filename: &str) -> Result<Vec<ZippedFile>, ZipValidity>
    {
        check_file_validity(filename)?;

	let mut list: Vec<ZippedFile> = vec![];

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
		let file_contents = unsafe {std::str::from_utf8_unchecked(&writer).to_string()};

                let zippedfile = ZippedFile::new(&file_path, file_size, Rc::new(file_contents));
                list.push(zippedfile);
	    }
	}

	Ok(list)
    }

    fn get_file_from_list<'a>(list: &'a [ZippedFile], filename: &str) -> &'a ZippedFile {
	let result = list.iter().find(|&f| f.path == filename).unwrap();
	result
    }

    fn get_sessions_total(list: &[ZippedFile]) -> (u32, usize) {
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
			},
			None => continue
		    }
		},
		None => continue
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
	    },
	    None => return value
	}

	value
    }

    //pub fn fiddler_saz_parse(fname: &str) -> Vec<FiddlerEntry> {
    pub fn fiddler_saz_parse(fname: &str) -> Result<Vec<FiddlerEntry>, ZipValidity> {
	let mut entries: Vec<FiddlerEntry> = vec![];

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

            let entry = FiddlerEntry::new(
                n, httpstatus, url, contentlength,
                &request_file_name, &response_file_name,
                &request_file.contents, &response_file.contents
            );

            entries.push(entry);
	}

	Ok(entries)
    }
}
