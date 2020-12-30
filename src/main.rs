use std::fs;
use std::io::BufReader;

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

fn zip_contents(filename: &str) -> zip::result::ZipResult<()>
{
	let fname = std::path::Path::new(filename);
    let file = fs::File::open(fname).unwrap();
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader).unwrap();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => {
                println!("Entry {} has a suspicious path", file.name());
                continue;
            }
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("Entry {} comment: {}", i, comment);
            }
        }

		if file.name().ends_with('/') {
            println!("Entry {} is a directory with name \"{}\"", i, outpath.display());
        } else {
            println!(
                "Entry {} is a file with name \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
        }
    }

	Ok(())
}

fn main() {
	let ftype = get_filetype("fiddler.saz");
	dbg!(ftype);
	// println!("{:?}", ftype);
	// zip_contents("fiddler.saz");
}
