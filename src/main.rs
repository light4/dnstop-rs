use std::{
    sync::{Arc, Mutex},
    thread,
};

use clap::Parser;
use color_eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    let opt = dnstop_rs::opt::Opt::parse();

    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dnstop=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let _conn = dnstop_rs::db::init_db("dnstop.db")?;
    let conn = Arc::new(Mutex::new(_conn));

    if !opt.noweb {
        let cloned_conn = Arc::clone(&conn);
        thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    dnstop_rs::web::run(opt.bind, cloned_conn).await.unwrap();
                });
        });
    }

    dnstop_rs::run(&opt, conn)?;
    Ok(())
}
