use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use std::fs;

#[derive(Debug, Deserialize)]
struct Surah {
    id: i32,
    city: String,
    name: Name,
    ayahs: i32,
    slug: String,
    translator: String,
}

#[derive(Debug, Deserialize)]
struct Name {
    translated: String,
    transliterated: String,
    codepoints: Vec<u32>,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = fs::read_dir("en").expect("FOLDER NOT FOUND");

    let pool = PgPoolOptions::new()
        .connect("postgres://alquran:alquran@127.0.0.1/alquran?currentSchema=alquran") // Update with your database credentials
        .await?;

    for file in files {
        let file = file.expect("unable to get file");
        let path = file.path();

        if path.is_file() && path.extension().map(|s| s == "json").unwrap_or(false) {
            let data = fs::read_to_string(&path).expect("Unable to read file");

            // Parse the JSON data
            let parsed: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");

            // Extract the Surah information
            let surah: Surah =
                serde_json::from_value(parsed[0].clone()).expect("Failed to parse Surah");

            // Print the Surah information
            println!("{:?}", surah);

            // Extract and print the Ayahs
            for ayah in parsed.as_array().unwrap().iter().skip(1) {
                println!("=======================");
                let ayah_data = ayah.as_array().expect("Ayah is not an array");
                let ayah_number =
                    ayah_data[0].as_u64().expect("Ayah number is not a number") as i32;
                let ayah_text = ayah_data[1].as_str().expect("Ayah text is not a string");
                println!("Ayah {}: {}", ayah_number, ayah_text);

                sqlx::query("UPDATE alquran_ayat SET en = $1 WHERE surah = $2 and ayat = $3")
                    .bind(ayah_text)
                    .bind(surah.id)
                    .bind(ayah_number)
                    .execute(&pool)
                    .await
                    .expect("Error updating Table");
                println!("SUCCESS INSERTING");
                println!("=======================");
            }
        }
    }
    // Read the JSON file

    Ok(())
}
