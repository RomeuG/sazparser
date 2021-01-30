sazparser
==========

A parser for SAZ file types generated by Fiddler.

Example
=======

```
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
```

License
=======

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Contribution
============

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
