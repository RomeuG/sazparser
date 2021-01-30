use std::env;

use sazparser;

fn main() {
    let args: Vec<String> = env::args().collect();

    // args[1] will be the file to parse
    let saz = sazparser::parse(&*args[1]);

    match saz {
        Ok(v) => {
            // use parsed information
            println!("{:?}", v);
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}
