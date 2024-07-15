use std::env;

use goat::URL;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: goat <url>");
        std::process::exit(1);
    } else {
        let url = URL::new(&args[1]);
        println!("url: {}", url);
    }
}
