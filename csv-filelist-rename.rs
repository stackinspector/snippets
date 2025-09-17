use std::collections::HashSet;

fn main() {
    let handle = std::fs::File::open(r"filelist_passed.csv").unwrap();
    let reader = csv::Reader::from_reader(handle);

    let mut src_filenames = HashSet::with_capacity(200);

    for item in std::fs::read_dir(r"final_files").unwrap() {
        let item = item.unwrap();
        assert!(item.file_type().unwrap().is_file());
        let orig_path = item.path();
        let src_filename = orig_path.file_name().unwrap().to_str().unwrap().to_owned();
        assert!(src_filenames.insert((src_filename, orig_path)));
    }

    let mut cnt1 = 0;
    let mut cnt2 = 0;

    for record in reader.into_records() {
        let record = record.unwrap();
        let mut record = record.iter();
        let name = record.next().unwrap();
        let filename_prefix = record.next().unwrap();
        assert!(matches!(record.next(), None));

        let illegal_chars = ['\u{2067}', '\u{2066}', '\u{202d}'];
        let filename_prefix = filename_prefix
            .replace(illegal_chars, "-")
            .replace(".MP3", ".mp3")
            .replace(".MOV", ".mov");

        cnt2 += 1;

        let found = src_filenames
            .iter()
            .find(|(src_filename, _)| src_filename.starts_with(filename_prefix.as_str()));

        if let Some((src_filename, orig_path)) = found {
            let mut dst_filename = r"passed_files\".to_owned();
            dst_filename.push_str(name);
            dst_filename.push_str("____");
            dst_filename.push_str(src_filename.as_str());
            cnt1 += 1;
            std::fs::copy(orig_path, dst_filename).unwrap();
        } else {
            dbg!(filename_prefix);
        }
    }

    dbg!(cnt1, cnt2);
}
