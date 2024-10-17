// Loads XML files from https://github.com/billyc999/Game-database-info/
// into a sqlite database.
use elementtree::Element;
use sqlx::{migrate::MigrateDatabase, Connection, SqliteConnection};

#[derive(Debug)]
struct GameRaw {
    name: String,
    description: String,
    rating: String,
    genre: String,
    developer: String, // dev
    publisher: String, // manufacturer
    year: String,
    platform: String
}

fn parse_games_from_xml_file(filename: &str) -> Vec<GameRaw> {
    let contents = std::fs::read_to_string(filename)
        .expect("Failed to open file");

    let root = Element::from_reader(contents.as_bytes())
        .expect("Failed to parse xml");

    let platform = root.find("header").unwrap().find("listname").unwrap().text().trim().to_string();

    root.find_all("game").map(|g| {
        GameRaw {
            name: g.get_attr("name").unwrap().trim().to_string(),
            description: g.find("description").unwrap().text().trim().to_string(),
            rating: g.find("rating").unwrap().text().trim().to_string(),
            genre: g.find("genre").unwrap().text().trim().to_string(),
            developer: g.find("dev").unwrap().text().trim().to_string(),
            publisher: g.find("manufacturer").unwrap().text().trim().to_string(),
            year: g.find("year").unwrap().text().trim().to_string(),
            platform: platform.clone()
        }
    }).collect()
}

#[tokio::main]
async fn main() {
    let db_url = "sqlite:games.db";

    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap() {
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    let mut conn = SqliteConnection::connect(&db_url).await
        .expect("Failed to connect to in memory sqlite");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS raw_games(
            name VARCHAR,
            description VARCHAR,
            rating VARCHAR,
            genre VARCHAR,
            developer VARCHAR,
            publisher VARCHAR,
            year VARCHAR,
            platform VARCHAR
    )").execute(&mut conn).await.expect("Failed to create table");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Need a filename to parse");
        return;
    }

    let games = parse_games_from_xml_file(&args[1]);

    for game in games {
        sqlx::query(
            "INSERT INTO raw_games
                (name,description,rating,genre,developer,publisher,year, platform)
                VALUES
                ($1,$2,$3,$4,$5,$6,$7,$8)
            "
        )
        .bind(game.name)
        .bind(game.description)
        .bind(game.rating)
        .bind(game.genre)
        .bind(game.developer)
        .bind(game.publisher)
        .bind(game.year)
        .bind(game.platform)
        .execute(&mut conn).await.expect("Failed to insert data");
    }
}