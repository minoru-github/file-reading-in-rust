use std::error::Error;
use std::{cell::RefCell, collections::BTreeMap, fs::File, io::*, result::Result};

type Type = i64;
type Frames = Vec<Option<i64>>;
type Data = BTreeMap<String, Frames>;

fn main() -> Result<(), Box<dyn Error>> {
    let rdr = read_with_path("./data/sample.csv")?;
    //let rdr = read_with_bytes()?;

    let data = parse(rdr)?;

    for (header, frame_data) in data {
        println!("[{}]", header);
        for e in frame_data {
            if let Some(e) = e {
                println!("{}", e);
            } else {
                println!("None");
            }
        }
    }

    Ok(())
}

#[allow(unused)]
fn read_with_path(path: &str) -> Result<csv::Reader<File>, Box<dyn Error>> {
    let rdr = csv::Reader::from_path(path)?;
    Ok(rdr)
}

#[allow(unused)]
fn read_with_bytes() -> Result<csv::Reader<&'static [u8]>, Box<dyn Error>> {
    let csv = "year,make,model,description
1948,Porsche,356,Luxury sports car
1967,Ford,Mustang fastback 1967,American car";
    let rdr = csv::Reader::from_reader(csv.as_bytes());
    Ok(rdr)
}

#[allow(unused)]
fn read_with_stdin() -> Result<csv::Reader<Stdin>, Box<dyn Error>> {
    let rdr = csv::Reader::from_reader(std::io::stdin());
    Ok(rdr)
}

fn parse<R: std::io::Read>(rdr: csv::Reader<R>) -> Result<Data, Box<dyn Error>> {
    // rdrからheadersとrecordsを生成するには可変参照が必要だが、rdr: &mutを引数にすると
    // headers()とrecords()で複数の可変参照が存在することになる。
    // &mutだとコンパイル時に可変参照のルールチェックがされるが、RefCellならば実行時にチェックされる。
    // RefCell使いつつ、複数の可変参照が無いように適切にdropさせれば実現可能。
    // https://doc.rust-jp.rs/book-ja/ch15-05-interior-mutability.html

    let rdr = RefCell::new(rdr);

    let mut map: Data = Data::new();
    let mut index_to_key = vec![];

    {
        // RefCellでbollow_mutしたrdrの可変参照をこのコードブロックでdropさせる
        let mut binding = rdr.borrow_mut();
        let headers = binding.headers()?;
        for header in headers.iter().map(|e| e.to_string()) {
            map.insert(header.clone(), Vec::new());
            index_to_key.push(header.clone());
        }
    }

    // RefCellを使えば、実行時に可変参照(RefMut<T>)の参照カウンタがゼロであればOK。
    // 実行時にも可変参照の参照カウンタがゼロじゃないならばpanic発動する。
    for res in rdr.borrow_mut().records() {
        let records = res?;
        for (index, record) in records.iter().enumerate() {
            let res = record.parse::<Type>();
            let record = if let Ok(record) = res {
                Some(record)
            } else {
                None
            };
            let key = index_to_key[index].to_string();
            map.entry(key).and_modify(|f| f.push(record));
        }
    }
    Ok(map)
}
