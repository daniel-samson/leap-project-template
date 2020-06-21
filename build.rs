use sass_rs::{compile_file, Options as SassOptions, OutputStyle};
use std::fs::{create_dir_all, read_to_string, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use toml::map::Map;
use toml::Value;

fn main() {
    dotenv::dotenv().ok();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Pack.toml");

    let pack_toml = get_pack_toml();
    let pack: Value = toml::from_str(pack_toml.as_str()).expect("Unable to parse Pack.toml");

    let paths = pack["sass"]
        .as_table()
        .expect("unable to read sass from Pack.toml");
    build_sass(paths);

    let files = pack["copy"]
        .as_table()
        .expect("unable to read copy from Pack.toml");
    copy_files(files);
}

fn get_pack_toml() -> String {
    read_to_string("Pack.toml").expect("Can't find Pack.toml")
}

fn copy_files(files: &Map<String, Value>) {
    for out_file in files.keys() {
        let in_files: Vec<String> = files
            .get(out_file)
            .expect("unable to find file")
            .as_array()
            .unwrap()
            .iter()
            .map(|in_file_value: &Value| -> String { in_file_value.as_str().unwrap().to_string() })
            .collect();

        for watch_file in in_files {
            println!("cargo:rerun-if-changed={}", watch_file.as_str());

            create_dir_all(
                Path::new(out_file)
                    .parent()
                    .expect("expected file but got a directory"),
            )
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

fn build_sass(paths: &Map<String, Value>) {
    for out_path in paths.keys() {
        let in_path = paths
            .get(out_path)
            .expect(format!("unable to get build path of {}", out_path).as_str())
            .as_str()
            .expect("expected sass build path to be a string");

        let mut options = SassOptions::default();
        options.output_style = OutputStyle::Compressed;
        compile_sass(
            &Path::new(in_path),
            &Path::new(out_path),
            "scss",
            &options.clone(),
        );

        options.indented_syntax = true;
        compile_sass(&Path::new(in_path), &Path::new(out_path), "sass", &options);
    }
}

fn compile_sass(sass_path: &Path, output_path: &Path, extension: &str, options: &SassOptions) {
    let pattern = format!("{}/**/*.{}", sass_path.display(), extension);
    let glob_files: Result<glob::Paths, glob::PatternError> = glob::glob(pattern.as_ref());

    let sass_files: Vec<PathBuf> = glob_files
        .expect("unable to glob files")
        .filter_map(|f: glob::GlobResult| -> Option<PathBuf> { f.ok() })
        .collect();

    for sass_file in &sass_files {
        println!("cargo:rerun-if-changed={}", sass_file.to_str().unwrap());
    }

    let sass_compile_files: Vec<PathBuf> = sass_files
        .into_iter()
        .filter(|file| {
            !file
                .as_path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with('_')
        })
        .collect();

    for file in sass_compile_files {
        let css = compile_file(&file, options.clone())
            .expect(format!("unable to compile {}", file.to_str().unwrap()).as_str());

        let path_inside_sass = file.strip_prefix(&sass_path).unwrap();
        let parent_inside_sass = path_inside_sass.parent();
        let css_output_path = output_path.join(path_inside_sass).with_extension("css");

        if parent_inside_sass.is_some() {
            create_dir_all(&css_output_path.parent().unwrap()).expect(
                format!(
                    "Unable to create {}",
                    &css_output_path.parent().unwrap().to_str().unwrap()
                )
                .as_str(),
            );
        }

        let out_file_handle = OpenOptions::new()
            .create(true)
            .truncate(false)
            .append(true)
            .open(Path::new(&css_output_path));

        out_file_handle
            .expect(
                format!(
                    "Unable to create/open {}",
                    css_output_path.to_str().unwrap()
                )
                .as_ref(),
            )
            .write_all(css.as_bytes())
            .expect(format!("Unable to build {}", css_output_path.to_str().unwrap()).as_ref());
    }
}
