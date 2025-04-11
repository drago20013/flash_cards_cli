mod db;
mod import;
mod learn;
mod models;
mod term;

use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the database
    let pool = db::init_db().await?;

    loop {
        term::clear_screen();
        println!("Welcome to Quizlet CLI!");
        println!("1. Import a new set");
        println!("2. Learn a set");
        println!("3. View statistics (coming soon)");
        println!("4. Set learning direction");
        println!("5. Exit");
        print!("Enter your choice (1-5): ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        match choice {
            "1" => import_set(&pool).await?,
            "2" => learn_set(&pool).await?,
            "3" => println!("Statistics feature coming soon!"),
            "4" => {
                term::clear_screen();
                println!("Choose learning direction:");
                println!("1. See term, guess definition");
                println!("2. See definition, guess term");
                print!("Enter your choice (1-2): ");
                io::stdout().flush()?;
                let mut dir_choice = String::new();
                io::stdin().read_line(&mut dir_choice)?;
                let dir_choice = dir_choice.trim();
                let direction = match dir_choice {
                    "1" => "term_to_definition",
                    "2" => "definition_to_term",
                    _ => {
                        println!("Invalid choice. Press Enter to try again...");
                        io::stdin().read_line(&mut String::new())?;
                        continue;
                    }
                };
                db::set_learning_direction(&pool, direction).await?;
                println!("Learning direction set to: {}", if direction == "term_to_definition" { "See term, guess definition" } else { "See definition, guess term" });
                print!("Press Enter to continue...");
                io::stdout().flush()?;
                io::stdin().read_line(&mut String::new())?;
            },
            "5" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid choice. Please enter a number between 1 and 4."),
        }
    }
    Ok(())
}

async fn learn_set(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    learn::learn_set(pool).await
}

async fn import_set(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    import::import_set(pool).await
}
