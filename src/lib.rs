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
	enum ZipValidity {
		Valid,
		Empty,
		Spanned,
		Invalid
	}

	#[derive(Debug, Clone)]
	struct ZippedFile {
		path: String,
		size: u64,
		contents: Rc<String>
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

	fn get_filetype(filename: &str) -> ZipValidity {
		let data = fs::read(filename).unwrap();

		let slice = &data[..4];
		if slice == constants::MAGIC_ZIP {
			return ZipValidity::Valid;
		} else if slice == constants::MAGIC_ZIP_EMPTY {
			return ZipValidity::Empty;
		} else if slice == constants::MAGIC_ZIP_SPANNED {
			return ZipValidity::Spanned;
		} else {
			return ZipValidity::Invalid;
		}
	}

	fn zip_contents(filename: &str) -> zip::result::ZipResult<Vec<ZippedFile>>
	{
		let mut list: Vec<ZippedFile> = vec![];

		let file = fs::File::open(filename).unwrap();
		let reader = BufReader::new(file);

		let mut archive = zip::ZipArchive::new(reader)?;

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

				list.push(ZippedFile {
					path: file_path,
					size: file_size,
					contents: Rc::new(file_contents)
				});
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

	pub fn fiddler_saz_parse(fname: &str) -> Vec<FiddlerEntry> {
		let mut entries: Vec<FiddlerEntry> = vec![];

		let mut list = zip_contents(&fname).unwrap();
		let (total_sessions, leading_zeroes) = get_sessions_total(&list);

		for n in 1..=total_sessions {
			let request_file_name = format!("raw/{:0fill$}_c.txt", n, fill = leading_zeroes);
			let response_file_name = format!("raw/{:0fill$}_s.txt", n, fill = leading_zeroes);

			let request_file = get_file_from_list(&list, &*request_file_name);
			let response_file = get_file_from_list(&list, &*response_file_name);

			let url = regex_get_url(&*request_file.contents);
			let httpstatus = regex_get_http_status(&*response_file.contents);
			let contentlength = regex_get_content_length(&*response_file.contents);

			// NOTE: if lossy is wanted
			// let request_contents_lossy = String::from_utf8_lossy(request_file.contents.as_bytes());
			// let response_contents_lossy = String::from_utf8_lossy(response_file.contents.as_bytes());

			entries.push(FiddlerEntry {
				index: n,
				result: httpstatus,
				url: url.to_string(),
				body: contentlength,
				file_request: request_file_name.clone(),
				file_response: response_file_name.clone(),
				file_request_contents: request_file.contents.clone(),
				file_response_contents: response_file.contents.clone()
			});
		}

		entries
	}
}
