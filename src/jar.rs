use std::io::{Read, Seek};
use std::path::Path;
use zip::result::ZipResult;
use crate::class::Class;

fn is_class_file(path: &str) -> bool {
    let path = Path::new(path);
    match path.extension() {
        Some(x) if x == "class" => true,
        _ => false
    }
}

pub fn load_jar<R: Read + Seek>(reader: R) -> ZipResult<()> {

    let mut zip = zip::ZipArchive::new(reader)?;
    for file_index in 0..zip.len()
    {
        let mut file = zip.by_index(file_index)?;
        if is_class_file(file.name()) {
            println!("Reading class {}", file.name());
            match Class::read(&mut file) {
                Ok(class) => (),
                Err(error) => println!("\t -> {:?}", error)
            }
        }
    }

    Ok(())
}