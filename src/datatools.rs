use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path};
use std::process::Command;
use std::str::FromStr;

pub fn read_value_from_file<P: AsRef<Path>, V: FromStr>(path: P, index: usize) -> Option<V> {
    let contents = fs::read_to_string(path)
        .unwrap_or(String::new());
    let values: Vec<&str> = contents.split(" ").collect();

    let value_str = match values.get(index) {
        Some(v) => v,
        None => "0",
    };
    
    match value_str.trim().parse() {
        Ok(v) => Some(v),
        Err(_) => None
    }
}

#[derive(Debug)]
pub struct FileSystemInfo {
    pub mounts: Vec<MountInfo>,
}

#[derive(Debug)]
pub struct MountInfo {
    pub name: String,
    pub size: f32,
    pub used: f32,
    pub available: f32,
    pub usage: u32,
    pub mount: String,
}

impl FileSystemInfo {
    pub fn get() -> Result<FileSystemInfo,Error> {
        let output = Command::new("df")
            .output()?;

        let utf8str = match std::str::from_utf8(output.stdout.as_slice()) {
            Ok(s) => s,
            Err(error) => {
                error!("Unexpected error while decoding df output: {}", error);
                let e = Error::new(ErrorKind::Other, "Could not decode df output");
                return Err(e);
            }
        };

        let mut info = FileSystemInfo {
            mounts: Vec::new()
        };

        let mut output_lines = utf8str.lines();
        output_lines.next(); // Skip header
        for line in output_lines {
            match parse_fs_mount_line(line) {
                Some(mount_info) => info.mounts.push(mount_info),
                None => {
                    // Ignore
                }
            }
        }

        Ok(info)
    }
}

fn parse_fs_mount_line(line: &str) -> Option<MountInfo> {
    let (name, size, used, available, usage, mount) = 
        scan_fmt!(line,
        "{} {f} {f} {f} {d}% {}",
        String, f32, f32, f32, u32, String);
    
    let divisor = 1.0 / 1024.0 / 1024.0;
    let size = size? * divisor;
    let used = used? * divisor;
    let available = available? * divisor;
    
    let info = MountInfo {
        name: name?,
        size: size,
        used: used,
        available: available,
        usage: usage?,
        mount: mount?
    };
    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_mount_line_can_parse() {
        let mount_info = parse_fs_mount_line("rootfs         243202148 181396036  61806112  75% /").unwrap();
        assert_eq!("rootfs", mount_info.name);
        assert_eq!(243202148.0 / 1024.0 / 1024.0, mount_info.size);
        assert_eq!(181396036.0 / 1024.0 / 1024.0, mount_info.used);
        assert_eq!(61806112.0 / 1024.0 / 1024.0, mount_info.available);
        assert_eq!(75, mount_info.usage);
        assert_eq!("/", mount_info.mount);
    }

    #[test]
    #[should_panic]
    fn bad_fs_mount_line_returns_none() {
        parse_fs_mount_line("rootfs         x y  61806112  75% /").unwrap();
    }
}
