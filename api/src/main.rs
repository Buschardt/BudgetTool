use std::path::PathBuf;

use budgettool_api::core::AppState;
use budgettool_api::core::db;
use clap::Parser;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Start the API server
    Serve,
    /// Seed a user into the database
    Seed {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => {
            let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
            let data_dir = std::env::var("DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/data/files"));
            let pool = db::init_pool(&database_url).await;
            let state = AppState {
                db: pool,
                jwt_secret,
                data_dir,
            };

            let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
            info!(addr = %listener.local_addr().unwrap(), "API listening");
            axum::serve(listener, budgettool_api::app(state))
                .await
                .unwrap();
        }
        Command::Seed { username, password } => {
            let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let pool = db::init_pool(&database_url).await;
            let hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).unwrap();
            sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
                .bind(&username)
                .bind(&hash)
                .execute(&pool)
                .await
                .expect("failed to insert user");
            info!(username = %username, "user created");
        }
    }
}
