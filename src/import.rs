use crate::term;
use sqlx::SqlitePool;
use std::fs::File;
use std::io::{self, BufReader};
use csv::ReaderBuilder;

pub async fn import_set(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // Prompt for set name
    term::clear_screen();
    println!("Enter the name of the set:");
    let mut set_name = String::new();
    io::stdin().read_line(&mut set_name)?;
    let set_name = set_name.trim();

    // Check if set already exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sets WHERE name = ?)")
        .bind(set_name)
        .fetch_one(pool)
        .await?;

    if exists {
        println!("Set '{}' already exists. Overwrite? (y/n)", set_name);
        let mut overwrite = String::new();
        io::stdin().read_line(&mut overwrite)?;
        if overwrite.trim().to_lowercase() != "y" {
            println!("Import cancelled.");
            return Ok(());
        }
        // Delete existing set and its terms
        let set_id: i64 = sqlx::query_scalar("SELECT id FROM sets WHERE name = ?")
            .bind(set_name)
            .fetch_one(pool)
            .await?;
        sqlx::query("DELETE FROM terms WHERE set_id = ?")
            .bind(set_id)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM sets WHERE id = ?")
            .bind(set_id)
            .execute(pool)
            .await?;
    }

    // Prompt for CSV file path
    println!("Enter the path to the CSV file:");
    let mut file_path = String::new();
    io::stdin().read_line(&mut file_path)?;
    let file_path = file_path.trim();

    // Open and read the CSV file
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new()
        .delimiter(b'^')
        .has_headers(false)
        .from_reader(reader);

    // Insert the new set into the database
    let set_id: i64 = sqlx::query_scalar("INSERT INTO sets (name) VALUES (?) RETURNING id")
        .bind(set_name)
        .fetch_one(pool)
        .await?;

    // Parse CSV and insert terms
    for result in csv_reader.records() {
        let record = result?;
        if record.len() != 2 {
            return Err("CSV must have exactly two columns: term and definition".into());
        }
        let term = &record[0];
        let definition = &record[1];
        sqlx::query("INSERT INTO terms (set_id, term, definition) VALUES (?, ?, ?)")
            .bind(set_id)
            .bind(term)
            .bind(definition)
            .execute(pool)
            .await?;
    }

    println!("Set '{}' imported successfully!", set_name);
    Ok(())
}
