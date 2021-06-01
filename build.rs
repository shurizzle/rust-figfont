use std::{
    env,
    fs::{canonicalize, read_dir, DirEntry, File},
    io::Write,
    path::Path,
};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("tests.rs");
    let mut test_file = File::create(&destination).unwrap();

    write_header(&mut test_file);

    let test_fonts = read_dir("./fonts/").unwrap();

    for font in test_fonts {
        write_test(&mut test_file, &font.unwrap());
    }
}

fn write_header(test_file: &mut File) {
    write!(
        test_file,
        r#"
use figfont::FIGfont;

        "#
    )
    .unwrap();
}

fn write_test(test_file: &mut File, entry: &DirEntry) {
    if entry.file_type().unwrap().is_file() && entry.path().extension().unwrap() == "flf" {
        let p = canonicalize(entry.path()).unwrap();
        let path = p.to_str().unwrap();
        let test_name = p.file_name().unwrap().to_str().unwrap();
        let test_name = &test_name[..(test_name.len() - 4)];

        write!(
            test_file,
            include_str!("./tests/test_template"),
            name = test_name,
            path = path
        )
        .unwrap();
    }
}
