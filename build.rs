use std::fs::{read_to_string, create_dir_all, File, OpenOptions};
use toml::Value;
use std::path::Path;
use std::io::{Write, Read};
use toml::map::Map;

fn main() {
    dotenv::dotenv().ok();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Pack.toml");

    let pack_toml = get_pack_toml();
    let pack: Value = toml::from_str(pack_toml.as_str())
        .expect("Unable to parse Pack.toml");

    let files = pack["copy"]
        .as_table()
        .expect("unable to read copy from Pack.toml");

    copy_files(files)
}

fn get_pack_toml() -> String {
    read_to_string("Pack.toml").expect("Can't find Pack.toml")
}

fn copy_files(files: &Map<String, Value>) {
    for out_file in files.keys() {
        let in_files: Vec<String> = files.get(out_file)
            .expect("unable to find file")
            .as_array()
            .unwrap()
            .iter()
            .map(|in_file_value: &Value| -> String { in_file_value.as_str().unwrap().to_string() })
            .collect();

        for watch_file in in_files {
            println!("cargo:rerun-if-changed={}", watch_file.as_str());

            create_dir_all(Path::new(out_file)
                .parent()
                .expect("expected file but got a directory"))
                .expect("unable to create directories");

            let in_file_buff = &mut vec![];
            File::open(watch_file.clone())
                .expect(format!("Unable to open {}", watch_file.clone()).as_ref())
                .read_to_end(in_file_buff)
                .expect(format!("Unable to read {}", watch_file.clone()).as_ref());


            let out_file_handle = OpenOptions::new()
                .create(true)
                .truncate(false)
                .append(true)
                .open(Path::new(out_file));

            out_file_handle
                .expect(format!("Unable to create/open {}", out_file).as_ref())
                .write_all(in_file_buff)
                .expect(format!("Unable to copy {} to {}", watch_file, out_file).as_ref());
        }
    }
}
