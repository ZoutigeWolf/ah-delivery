use crate::models::Shift;
use std::env;
use std::sync::LazyLock;
use tokio_postgres::{Error, NoTls, Row};

static POSTGRES_CREDENTIALS: LazyLock<String> = LazyLock::new(|| {
    env::var("POSTGRES_CREDENTIALS").expect("POSTGRES_CREDENTIALS must be set in the environment")
});

pub async fn upload_shift(shift: &Shift) -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(&POSTGRES_CREDENTIALS, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    client
        .execute(
            "
        INSERT INTO shifts (boff_id, name, date, planning, start, \"end\", info)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (date, planning, boff_id)
        DO UPDATE SET
            name = EXCLUDED.name,
            start = EXCLUDED.start,
            \"end\" = EXCLUDED.\"end\",
            info = EXCLUDED.info;
        ",
            &[
                &shift.boff_id,
                &shift.name,
                &shift.date,
                &shift.planning,
                &shift.start,
                &shift.end,
                &shift.info,
            ],
        )
        .await?;

    Ok(())
}

pub async fn fetch_shifts() -> Result<Vec<Shift>, Error> {
    let (client, connection) = tokio_postgres::connect(&POSTGRES_CREDENTIALS, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT * FROM shifts", &[])
        .await?
        .iter()
        .map(Shift::from_row)
        .collect::<Vec<Shift>>();

    Ok(rows)
}

impl Shift {
    fn from_row(row: &Row) -> Shift {
        Shift {
            boff_id: row.get("boff_id"),
            name: row.get("name"),
            date: row.get("date"),
            planning: row.get("planning"),
            start: row.get("start"),
            end: row.get("end"),
            info: row.get("info"),
        }
    }
}
