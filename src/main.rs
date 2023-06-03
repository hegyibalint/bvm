use std::fs::File;
use std::io;

use clap::Parser;

use crate::class::Class;
use crate::packaging::jar;

mod class;
mod packaging;
mod vm;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Colon separated path of classes
    // #[clap(short, long)]
    // classpath: Option<String>,
    /// Main class to be executed
    main_class: String,
}

fn main() {
    // let args = Args::parse();

    // let files = [
    //     "/home/baprof/Downloads/rt11jar/java.desktop/com/sun/beans/editors/ByteEditor.class",
    //     "/home/baprof/Downloads/rt11jar/java.desktop/com/sun/beans/editors/ColorEditor.class",
    // ];

    // for file in files {
    //     let class_file = File::open(file).unwrap();
    //     let mut class_reader = io::BufReader::new(class_file);
    //     println!("Reading class {}", file);
    //     match Class::read(&mut class_reader) {
    //         Ok(class) => (),
    //         Err(error) => println!("\t -> {:?}", error),
    //     }
    // }

    let rt_jar_file = File::open("/Users/bhegyi/.sdkman/candidates/java/8.0.372-zulu/zulu-8.jdk/Contents/Home/jre/lib/rt.jar").unwrap();
    let rt_jar_reader = io::BufReader::new(rt_jar_file);
    jar::load_jar(rt_jar_reader);

    let main_class_file = File::open("res/Main.class").unwrap();
    let mut main_class_reader = io::BufReader::new(main_class_file);

    let main_class = Class::read(&mut main_class_reader).unwrap();
    println!("{:#?}", main_class);
}
