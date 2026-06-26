use std::fs::File;
use std::io;
use std::io::BufReader;

use clap::Parser;

use crate::class::Class;
use crate::packaging::jar::Jar;

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

    // let mut jar = Jar::new("/Users/bhegyi/.sdkman/candidates/java/8.0.372-zulu/zulu-8.jdk/Contents/Home/jre/lib/rt.jar").unwrap();
    // for class_path in ["java/lang/Object", "java/lang/String"] {
    //     let mut class_file = jar.open(class_path).unwrap();
    //     let object_class = Class::read(&mut class_file).unwrap();
    //     // println!("{:#?}", object_class);
    //
    //     let this_class = object_class.this_class;
    //     let class_ref = &object_class.constant_pool[this_class];
    //     println!("{:#?}", class_ref);
    //
    //     match class_ref {
    //         Constant::Class(const_class) => {
    //             let const_class = &object_class.constant_pool[const_class.name_index];
    //             println!("{:#?}", const_class);
    //         }
    //         _ => panic!("Not a class reference"),
    //     }
    // }

    // let rt_jar_file = File::open("/Users/bhegyi/.sdkman/candidates/java/8.0.372-zulu/zulu-8.jdk/Contents/Home/jre/lib/rt.jar").unwrap();
    // let rt_jar_reader = BufReader::new(rt_jar_file);
    let mut rt_jar = Jar::new("/Users/bhegyi/.sdkman/candidates/java/8.0.372-zulu/zulu-8.jdk/Contents/Home/jre/lib/rt.jar").unwrap();
    //
    // let main_class_file = File::open("res/Add.class").unwrap();
    // let mut main_class_reader = BufReader::new(main_class_file);

    let main_class = Class::read(&mut rt_jar.open("java.lang.Object").unwrap()).unwrap();
    println!("{}", main_class);
}
