use std::{env, fs};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    let path_to_dir = Path::new("src").join("actions");
    let dir = fs::read_dir(&path_to_dir).unwrap();
    let actions: Vec<String> = dir
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.metadata().map(|m| m.is_file()).unwrap_or_default())
        .filter_map(|entry| entry.file_name().to_str().map(String::from))
        .filter(|file_name| {
            let path_to_file = path_to_dir.join(file_name);
            let file = File::open(&path_to_file).unwrap();
            BufReader::new(file).lines()
                .filter_map(|line| line.ok())
                .find(|line| line.find("#[define_action]").is_some())
                .is_some()
        })
        .collect();
    println!("{:?}", actions);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("action_impl_registry.rs");
    let file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(dest_path)
        .unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all("use lazy_static::lazy_static;\n\n".as_bytes()).unwrap();
    writer.write_all("lazy_static!{\n".as_bytes()).unwrap();
    writer.write_all("\tpub static ref REGISTRY: Vec<crate::actions::action_impl::ActionImpl> = {\n".as_bytes()).unwrap();
    writer.write_all("\t\tvec![\n".as_bytes()).unwrap();

    for action in actions {
        let name = Path::new(&action).file_stem().and_then(|s| s.to_str()).unwrap();
        writer.write_fmt(format_args!("\t\t\tcrate::actions::{}::INSTANCE,\n", name)).unwrap();
    }

    writer.write_all("\t\t]\n".as_bytes()).unwrap();
    writer.write_all("\t};\n".as_bytes()).unwrap();
    writer.write_all("}\n".as_bytes()).unwrap();
}