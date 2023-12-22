use std::io::{Write, Read};
use std::{fs::File, path::Path};

use parquet::data_type::AsBytes;
use parquet::file::reader::SerializedFileReader;
use random_email_sender::email_format::{Email, OriginalEmail};
use xz2::read::XzEncoder;

fn main() {
    let path = Path::new("emails.parquet.brotli");
    if let Ok(file) = File::open(path) {
        let reader = SerializedFileReader::new(file).unwrap();

        // Start on first row
        let mut emails = vec![];
        let mut file_num = 0;
        for (i, row) in reader.into_iter().enumerate() {
            let original_email: OriginalEmail = row.expect("Error reading row").into();
            let parsed_email: Email = original_email.into();

            emails.push(parsed_email);

            if i > 0 && i % 1000 == 0 {
                println!("{}", i);
                file_num += 1;
                write_emails(file_num, &mut emails);
            }
        }
        file_num += 1;
        write_emails(file_num, &mut emails);
    }
}

fn write_emails(file_num: usize, emails: &mut Vec<Email>) {
    let emails_serialized = serde_json::to_string_pretty(&emails).unwrap();

    let mut encoder = XzEncoder::new(emails_serialized.as_bytes(), 9);
    let mut buf = vec![];
    encoder.read_to_end(&mut buf).expect("Unable to compress email.");

    let path = format!("emails/{}.json.xz", file_num);
    let mut file = File::create(path).unwrap();
    file.write_all(buf.as_bytes()).unwrap();

    emails.clear();
}
