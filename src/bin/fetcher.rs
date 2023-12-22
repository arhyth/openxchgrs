use chrono::{Datelike, TimeZone, Timelike, Utc};
use clap;
use clap::Parser;
use reqwest::{Client, Method, Request, Url};
use serde_json;
use sqlx::{postgres::PgConnection, Connection, Row};

use rates::Rate;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Config {
    /// Postgres instance
    #[arg(short = 'n', long, default_value = "openexchangerates")]
    db_name: String,

    /// Postgres host or "location"
    #[arg(short = 'l', long)]
    db_loc: String,

    /// Postgres port
    #[arg(short = 'p', long, default_value = "5432")]
    db_port: String,

    /// User to log in
    #[arg(short = 'u', long)]
    db_user: String,

    /// User password
    #[arg(short = 'w', long)]
    db_pwd: String,

    /// OpenExchangeRates rates table
    #[arg(short = 't', long, default_value = "rates")]
    oxr_rates_table: String,

    /// OpenExchangeRates endpoint
    #[arg(
        short = 'o',
        default_value = "https://openexchangerates.org/api/latest.json"
    )]
    oxr_endpoint: String,

    /// OpenExchangeRates API key
    #[arg(short = 'k', long)]
    oxr_app_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::parse();
    let dsn = std::format!(
        "postgres://{}:{}@{}:{}/{}",
        cfg.db_user,
        cfg.db_pwd,
        cfg.db_loc,
        cfg.db_port,
        cfg.db_name
    );
    let mut conn = PgConnection::connect(&dsn).await?;

    let utc_now = Utc::now();
    let utc_now_hr = Utc.with_ymd_and_hms(
        utc_now.year(),
        utc_now.month(),
        utc_now.day(),
        utc_now.hour(),
        0,
        0,
    );

    let rt = cfg.oxr_rates_table.as_str();
    let fmts = format!("INSERT INTO {} (timestamp, base) VALUES ($1, $2) RETURNING id;", rt);
    let sql: &str = fmts.as_str();
    let placehold = sqlx::query(sql)
        .bind(utc_now_hr.unwrap().timestamp())
        .bind("USD")
        .fetch_one(&mut conn)
        .await
        .inspect_err(|e| eprintln!("failed writing placeholder row: {e}"))?;

    let placehold_id: i64 = placehold.get(0);
    println!("Inserted placeholder row `{}`", placehold_id);
    let url = Url::parse_with_params(cfg.oxr_endpoint.as_str(), &[("app_id", cfg.oxr_app_id)])?;
    let client = Client::new();
    let req = Request::new(Method::GET, url);
    let rate = client.execute(req).await?.json::<Rate>().await?;

    let pairs_json = serde_json::to_value(rate.pairs)?;
    let fmtd = format!("UPDATE {} SET pairs = $1 WHERE id = $2", rt);
    let usql = fmtd.as_str();
    sqlx::query(usql)
        .bind(pairs_json)
        .bind(placehold_id)
        .execute(&mut conn)
        .await
        .inspect_err(|e| eprintln!("failed filling row `{placehold_id}`: {e}"))?;

    Ok(())
}
