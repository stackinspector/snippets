use csv::WriterBuilder;

fn main() {
    let handle = std::fs::File::open("names.csv").unwrap();
    let reader = csv::Reader::from_reader(handle);
    let (mut nickname, mut qq_name) = reader
        .into_records()
        .map(|it| it.unwrap())
        .map(|it| it.iter().map(str::to_string).collect::<Vec<_>>())
        .map(|it| ((it[0].clone(), false), (it[1].clone(), false)))
        .unzip::<_, _, Vec<_>, Vec<_>>();

    for idx in 0..nickname.len() {
        let it = nickname[idx].0.clone();
        let duplicated = nickname
            .iter()
            .enumerate()
            .any(|(i, (v, _))| i != idx && it == *v);
        nickname[idx].1 = duplicated;
    }

    for idx in 0..qq_name.len() {
        let it = qq_name[idx].0.clone();
        let duplicated = qq_name
            .iter()
            .enumerate()
            .any(|(i, (v, _))| i != idx && it == *v);
        qq_name[idx].1 = duplicated;
    }

    let result = nickname
        .iter()
        .zip(qq_name.iter())
        .map(|(a, b)| a.1 == b.1)
        .collect::<Vec<_>>();

    let mut writer = WriterBuilder::new().from_writer(vec![]);
    for ((nickname, nickname_dup), ((qq_name, qq_name_dup), dup_eq)) in
        nickname.iter().zip(qq_name.iter().zip(result))
    {
        writer
            .write_record([
                nickname,
                &nickname_dup.to_string(),
                qq_name,
                &qq_name_dup.to_string(),
                &dup_eq.to_string(),
            ])
            .unwrap();
    }

    std::fs::write("out.csv", writer.into_inner().unwrap()).unwrap();
}
