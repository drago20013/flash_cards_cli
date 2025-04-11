use rand::seq::SliceRandom;
use std::io::{self, Write};
use crate::term;
use crate::db::get_learning_direction;


pub async fn learn_set(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // Select a set
    let set_id = select_set(pool).await?;
    let direction = get_learning_direction(pool).await?;
    // Initialize or resume the session
    init_or_resume_session(pool, set_id).await?;

    loop {
        // Get unmastered terms
        let mut terms = get_unmastered_terms(pool, set_id).await?;
        if terms.is_empty() {
            println!("Congratulations! You've mastered all terms in this set.");
            // Clear the session
            sqlx::query("DELETE FROM sessions WHERE set_id = ?")
                .bind(set_id)
                .execute(pool)
                .await?;
            break;
        }

        // Shuffle terms for variety
        terms.shuffle(&mut rand::rng());

        // Present each term
        for (term_id, term, definition) in terms {
            term::clear_screen();
            if direction == "term_to_definition" {
                println!("Term: {}", term);
                print!("Enter the definition: ");
            } else {
                println!("Definition: {}", definition);
                print!("Enter the term: ");
            }
            io::stdout().flush()?;
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            let answer = answer.trim();

            //FIXME: change to ctrl+q or sth, "exit" might be actual word to guess
            if answer.to_lowercase() == "exit" {
                println!("Exiting learning session.");
                return Ok(());
            }

            let correct = if direction == "term_to_definition" {
                answer.to_lowercase() == definition.to_lowercase()
            } else {
                answer.to_lowercase() == term.to_lowercase()
            };

            if correct {
                println!("Correct!");
                // Mark the term as mastered
                sqlx::query("UPDATE sessions SET mastered = 1 WHERE set_id = ? AND term_id = ?")
                    .bind(set_id)
                    .bind(term_id)
                    .execute(pool)
                    .await?;
            } else {
                if direction == "term_to_definition" {
                    println!("Incorrect. The correct definition is: {}", definition);
                } else {
                    println!("Incorrect. The correct term is: {}", term);
                }
            }
            print!("Press Enter to continue...");
            io::stdout().flush()?;
            io::stdin().read_line(&mut String::new())?;
        }
    }

    Ok(())
}

async fn get_unmastered_terms(pool: &sqlx::SqlitePool, set_id: i64) -> Result<Vec<(i64, String, String)>, sqlx::Error> {
    let terms: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT t.id, t.term, t.definition
         FROM terms t
         JOIN sessions s ON t.id = s.term_id AND s.set_id = ?
         WHERE s.mastered = 0"
    )
    .bind(set_id)
    .fetch_all(pool)
    .await?;

    Ok(terms)
}

async fn init_or_resume_session(pool: &sqlx::SqlitePool, set_id: i64) -> Result<(), sqlx::Error> {
    // Check if a session exists for this set
    let session_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sessions WHERE set_id = ?)"
    )
    .bind(set_id)
    .fetch_one(pool)
    .await?;

    if !session_exists {
        // Create a new session by inserting all terms with mastered = 0
        sqlx::query(
            "INSERT INTO sessions (set_id, term_id, mastered)
             SELECT ?, id, 0 FROM terms WHERE set_id = ?"
        )
        .bind(set_id)
        .bind(set_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn select_set(pool: &sqlx::SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    // Fetch all sets from the database
    let sets: Vec<(i64, String)> = sqlx::query_as("SELECT id, name FROM sets")
        .fetch_all(pool)
        .await?;

    if sets.is_empty() {
        println!("No sets available. Please import a set first.");
        return Err("No sets found".into());
    }

    // Display sets to the user
    println!("Available sets:");
    for (i, (_, name)) in sets.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }

    // Get user input
    print!("Enter the number of the set you want to learn: ");
    io::stdout().flush()?;
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice: usize = choice.trim().parse()?;

    if choice < 1 || choice > sets.len() {
        println!("Invalid choice.");
        return Err("Invalid set number".into());
    }

    let set_id = sets[choice - 1].0;
    Ok(set_id)
}
