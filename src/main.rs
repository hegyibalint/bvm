use std::fs::File;
use std::io;

use clap::Parser;
use crate::class::Class;

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

    let files = [
        "/home/baprof/Downloads/rt11jar/java.desktop/com/sun/beans/editors/ByteEditor.class",
        "/home/baprof/Downloads/rt11jar/java.desktop/com/sun/beans/editors/ColorEditor.class",
    ];

    for file in files {
        let class_file = File::open(file).unwrap();
        let mut class_reader = io::BufReader::new(class_file);
        println!("Reading class {}", file);
        match Class::read(&mut class_reader) {
            Ok(class) => (),
            Err(error) => println!("\t -> {:?}", error)
        }
    }

    let jar_file = File::open("/home/baprof/Developer/Coding/BVM-Projects/build/libs/bvm-projects-1.0-SNAPSHOT.jar").unwrap();
    let jar_file = File::open("/home/baprof/Downloads/jdk8u322-b06/jre/lib/rt.jar").unwrap();
    let jar_reader = io::BufReader::new(jar_file);
    jar::load_jar(jar_reader);
}