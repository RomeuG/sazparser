use std::fs;
use std::io::BufReader;

use select::predicate::Predicate;

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
    contents: String
}

#[derive(Debug)]
struct FiddlerEntry {
    index: u32,
    result: u32,
    protocol: String,
    host: String,
    url: String,
    body: u32,
    caching: String,
    contenttype: String,
    process: String,
    comments: String,
    custom: String,
    file_request: String,
    file_response: String
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
            std::io::copy(&mut zipped_file, &mut writer);
            let file_contents = std::str::from_utf8(&writer).unwrap().to_string();

            list.push(ZippedFile {
                path: file_path,
                size: file_size,
                contents: file_contents
            });
        }
    }

    Ok(list)
}

fn get_file_from_list<'a>(list: &'a [ZippedFile], filename: &str) -> &'a ZippedFile {
    let result = list.iter().find(|&f| f.path == filename).unwrap();
    result
}

// fn get_file_from_list(list: &[ZippedFile], filename: &str) -> ZippedFile {
//     let result = list.iter().find(|&f| f.path == filename).unwrap();
//     result.clone()
// }

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

fn main() {
    let list = zip_contents("fiddler.saz").unwrap();
    let result = get_file_from_list(&list, "_index.htm");

    let doc = select::document::Document::from(result.contents.as_str());
    let table = doc.find(select::predicate::Name("tr"));

    for (index, tr) in table.into_iter().enumerate() {
        if index == 0 {
            continue
        }

        let _frequest = tr.children().nth(0).unwrap()
            .children().nth(0).unwrap()
            .attr("href").unwrap();

        let _fresponse = tr.children().nth(0).unwrap()
            .children().nth(2).unwrap()
            .attr("href").unwrap();

        let _idx = tr.children().nth(1).unwrap().text();
        let _result = tr.children().nth(2).unwrap().text();
        let _protocol = tr.children().nth(3).unwrap().text();
        let _host = tr.children().nth(4).unwrap().text();
        let _url = tr.children().nth(5).unwrap().text();
        let mut _body = tr.children().nth(6).unwrap().text();
        let _caching = tr.children().nth(7).unwrap().text();
        let _contenttype = tr.children().nth(8).unwrap().text();
        let _process = tr.children().nth(9).unwrap().text();
        let _comments = tr.children().nth(10).unwrap().text();
        let _custom = tr.children().nth(11).unwrap().text();

        // sanitize whatever needs sanitization
        remove_whitespace(&mut _body);

        let omegalul = FiddlerEntry {
            index: _idx.parse::<u32>().unwrap(),
            result: _result.parse::<u32>().unwrap(),
            protocol: _protocol,
            host: _host,
            url: _url,
            body: _body.parse::<u32>().unwrap(),
            caching: _caching,
            contenttype: _contenttype,
            process: _process,
            comments: _comments,
            custom: _custom,
            file_request: _frequest.to_string(),
            file_response: _fresponse.to_string()
        };

        dbg!(omegalul);
    }
}
