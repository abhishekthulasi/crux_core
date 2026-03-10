use crate::db::DbContext;
use crate::models::Turn;
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

// Modern, standard library way to hold a global DB instance
static DB: OnceLock<Mutex<DbContext>> = OnceLock::new();

pub fn init_db(db_path: String) -> anyhow::Result<()> {
    // 1. Check if the database is already initialized.
    // If it is, we just return Ok(()) early so Flutter's Hot Reload doesn't crash.
    if DB.get().is_some() {
        // You can add a print statement here if you want to track hot reloads
        println!("Database already initialized. Skipping.");
        return Ok(());
    }

    // 2. Not initialized yet, so let's set it up.
    let context =
        DbContext::new(&db_path).map_err(|e| anyhow::anyhow!("Failed to init DB: {}", e))?;

    // 3. We use `let _ = ` to swallow the error just in case two threads
    // tried to initialize it at the exact same microsecond.
    let _ = DB.set(Mutex::new(context));

    Ok(())
}

pub fn send_message(parent_id: Option<String>, content: String) -> anyhow::Result<Vec<Turn>> {
    let db_mutex = DB
        .get()
        .expect("Database not initialized! Call init_db first.");
    let db = db_mutex.lock().unwrap();

    // 1. Create and insert User Turn
    let user_turn = Turn {
        id: Uuid::new_v4().to_string(),
        parent_turn_id: parent_id.clone(),
        role: "user".to_string(),
        content: content,
        created_at: String::new(),
    };
    db.insert_turn(&user_turn).unwrap(); // In production, map this error

    // 2. Simulate MVP 2 Dummy Bot Response
    let bot_turn = Turn {
        id: Uuid::new_v4().to_string(),
        parent_turn_id: Some(user_turn.id.clone()),
        role: "assistant".to_string(),
        content: "This is a dummy response for MVP 2.".to_string(),
        created_at: String::new(),
    };
    db.insert_turn(&bot_turn).unwrap();

    // 3. Return the new active branch
    let branch = db.get_chat_branch(&bot_turn.id).unwrap();
    Ok(branch)
}

pub fn fetch_branch(active_turn_id: String) -> anyhow::Result<Vec<Turn>> {
    let db_mutex = DB.get().expect("Database not initialized!");
    let db = db_mutex.lock().unwrap();

    let branch = db.get_chat_branch(&active_turn_id).unwrap();
    Ok(branch)
}
