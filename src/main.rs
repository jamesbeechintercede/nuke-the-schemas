use sqlx::{Execute, PgPool, Postgres, QueryBuilder};
use sqlx::postgres::PgPoolOptions;
use serde_json;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    println!("Nuke the schemas!");

    let schema_name: String = get_schema_name();
    let connection_strings: Vec<String> = get_connection_strings();

    for i in 0..connection_strings.len() {
        let connection_string = &connection_strings[i];
        println!("Connecting to database: {}", connection_string);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .expect("Failed to connect to database");
        drop_schema(&pool, schema_name.clone()).await.expect("Failed to drop schema");
        create_schema(&pool, schema_name.clone()).await.expect("Failed to create schema");
    }
}

// Helper function to locate settings.json next to the running executable
fn get_settings_path() -> PathBuf {
    let mut exe_path = std::env::current_exe()
        .expect("Failed to get current executable path");
    exe_path.pop();
    exe_path.push("settings.json");

    // Check if the file exists; if not, create a default template
    if !exe_path.exists() {
        let default_settings = r#"{
  "connection_strings": [
    ""
  ],
  "schema_name": ""
}"#;

        fs::write(&exe_path, default_settings)
            .expect("Failed to automatically create default settings.json file");

        println!("Created a new default settings.json at {:?}", exe_path);
        println!("Please edit this file with your database configurations and run the program again.");
        std::process::exit(0);
    }

    exe_path
}

fn get_connection_strings() -> Vec<String> {
    let path = get_settings_path();
    let settings_json = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read settings.json at {:?}", path));

    // Strip BOM marker if present
    let settings_json = settings_json.trim_start_matches('\u{FEFF}');
    let settings: serde_json::Value = serde_json::from_str(settings_json)
        .expect("Failed to parse settings.json");

    let connection_strings = settings["connection_strings"].as_array().unwrap();
    let mut result = Vec::new();
    for connection_string in connection_strings {
        result.push(connection_string.as_str().unwrap().to_string());
    }
    result
}

fn get_schema_name() -> String {
    let path = get_settings_path();
    let settings_json = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read settings.json at {:?}", path));

    let settings_json = settings_json.trim_start_matches('\u{FEFF}');
    let settings: serde_json::Value = serde_json::from_str(settings_json)
        .expect("Failed to parse settings.json");
    settings["schema_name"].as_str().unwrap().to_string()
}

async fn drop_schema(pool: &PgPool, schema_name: String) -> Result<(), sqlx::Error> {
    println!("Dropping schema: {}", schema_name);
    let mut query_builder = QueryBuilder::<Postgres>::new("DROP SCHEMA IF EXISTS ");
    query_builder.push("\"").push(schema_name).push("\"");
    query_builder.push(" CASCADE");

    let sql = query_builder.build().sql();
    sqlx::query(sql).execute(pool).await?;
    Ok(())
}

async fn create_schema(pool: &PgPool, schema_name: String) -> Result<(), sqlx::Error> {
    println!("Creating schema: {}", schema_name);
    let mut query_builder = QueryBuilder::<Postgres>::new("CREATE SCHEMA IF NOT EXISTS ");
    query_builder.push("\"").push(schema_name).push("\"");

    let sql = query_builder.build().sql();
    sqlx::query(sql).execute(pool).await?;
    Ok(())
}
