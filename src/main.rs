use std::env;

use goat::Url;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let url = Url::new(&args[1]);
        println!("url: {}", url);
    } else {
        println!("usage: goat <url>");
        std::process::exit(1);
    }
}
