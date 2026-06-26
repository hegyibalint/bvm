use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

use zip::read::ZipFile;
use zip::result::{ZipError, ZipResult};
use zip::ZipArchive;

use crate::class::Class;
use crate::packaging::ClassReadingError;

pub struct Jar {
    path: Box<Path>,
    archive: ZipArchive<File>,
}

impl Jar {
    pub fn new(path: &str) -> Result<Jar, ZipError> {
        let path = Path::new(path);
        let reader = File::open(path)?;

        Ok(Jar {
            path: Box::from(path),
            archive: ZipArchive::new(reader)?,
        })
    }

    pub fn open(&mut self, fqn: &str) -> Result<ZipFile, ClassReadingError> {
        Ok(self
            .archive
            .by_name(&format!("{}.class", fqn.replace(".", "/")))?)
    }
}

impl From<ZipError> for ClassReadingError {
    fn from(value: ZipError) -> Self {
        ClassReadingError::ZipError(value)
    }
}

fn is_class_file(path: &str) -> bool {
    let path = Path::new(path);
    match path.extension() {
        Some(x) if x == "class" => true,
        _ => false,
    }
}

pub fn load_jar<R: Read + Seek>(reader: R) -> ZipResult<()> {
    let mut zip = ZipArchive::new(reader)?;
    for file_index in 0..zip.len() {
        let mut file = zip.by_index(file_index)?;
        if is_class_file(file.name()) {
            println!("Reading class {}", file.name());
            match Class::read(&mut file) {
                Ok(_) => (),
                Err(error) => println!("\t -> {:?}", error),
            }
        }
    }

    Ok(())
}
