use std::{
    fs::OpenOptions,
    io::Write,
};

pub fn log(message: impl AsRef<str>) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ziro.log")
        .unwrap();

    writeln!(file, "{}", message.as_ref()).ok();
}