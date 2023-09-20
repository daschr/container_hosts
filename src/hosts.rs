use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Error as IoError;
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub struct Hosts {
    file: PathBuf,
    pub sections: HashMap<Option<String>, Vec<String>>,
}

impl Hosts {
    pub fn new<P: Into<PathBuf> + AsRef<Path>>(path: P) -> Result<Self, IoError> {
        let path: PathBuf = path.into();
        let fd = OpenOptions::new()
            .read(true)
            .write(true) // just opening it rw here to check, if we can do it
            .open(path.clone())?;
        let mut sections: HashMap<Option<String>, Vec<String>> = HashMap::new();

        let mut current_section: Option<String> = None;
        let mut reader = BufReader::new(fd);
        let mut line = String::new();

        let mut entries: Vec<String> = Vec::new();

        while reader.read_line(&mut line)? != 0 {
            let tr_line = line.trim();

            if tr_line.starts_with('#')
                && tr_line
                    .trim_start_matches(['#', ' '])
                    .starts_with("SECTION")
            {
                let new_section_name = match tr_line.split(' ').last() {
                    Some(name) if name != "SECTION" => name,
                    _ => continue,
                };

                if !entries.is_empty() {
                    sections.insert(current_section, entries.clone());
                    entries.clear();
                }

                current_section = Some(String::from(new_section_name));
            } else {
                entries.push(tr_line.to_string());
            }

            line.clear();
        }

        if !entries.is_empty() {
            sections.insert(current_section, entries);
        }

        Ok(Hosts {
            file: path,
            sections,
        })
    }

    pub fn update_section<S: Into<String>>(
        &mut self,
        section_name: Option<S>,
        entries: Vec<String>,
    ) {
        self.sections
            .insert(section_name.map(|v| v.into()), entries);
    }

    pub fn write(&mut self) -> Result<(), IoError> {
        let fd = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.file.clone())?;
        let mut writer = BufWriter::new(fd);

        if let Some(entries) = self.sections.remove(&None) {
            for e in entries.iter() {
                writer.write_all(e.as_bytes())?;
                writer.write_all(&[b'\n'])?;
            }
        }

        for (section, entries) in self.sections.iter() {
            let section = section.as_ref().unwrap();
            writer.write_all(format!("# SECTION {}\n", section).as_bytes())?;

            for e in entries.iter() {
                writer.write_all(e.as_bytes())?;
                writer.write_all(&[b'\n'])?;
            }
        }

        writer.flush()?;

        Ok(())
    }
}
