use std::fs::File;
use std::io;

use clap::Parser;

mod jar;
mod class;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Colon separated path of classes
    // #[clap(short, long)]
    // classpath: Option<String>,

    /// Main class to be executed
    main_class: String
}

fn main() {
    let args = Args::parse();

    let main_class_file = File::open(args.main_class).unwrap();
    let mut main_class_reader = io::BufReader::new(main_class_file);


    let class = class::Class::read(&mut main_class_reader).unwrap();
    println!("{:#?}", class);
}
