use axum::{routing, Router, serve};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utils::handlers::{contact_me, homepage, forward_notes_to_telegram};


#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    let website = tokio::spawn(website_router());
    let telegram = tokio::spawn(telegram_bot());

    let (web,tg) = tokio::join!(website, telegram);

    if let Err(e) = web {
        info!("Error: {:?}", e);
    }

    if let Err(e) = tg {
        info!("Error: {:?}", e);
    }

    Ok(())
}



async fn website_router() -> anyhow::Result<(), anyhow::Error> {

    info!("Iniciando website...");

    // Path to static files
    let src_path = std::env::current_dir()?.join("public");

    let static_files = Router::new()
        .nest_service(
            "/styles",
            ServeDir::new(format!("{}/styles", src_path.to_str().unwrap())),
        )
        .nest_service(
            "/js",
            ServeDir::new(format!("{}/js", src_path.to_str().unwrap())),
        )
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", src_path.to_str().unwrap())),
        );

    let router = Router::new()
        .route("/", routing::get(homepage))
        .route("/contact", routing::post(contact_me))
        .nest("/", static_files);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6900").await?;

    info!("Servidor listo - http://localhost:6900");
    serve(listener, router).await?;
    Ok(())
}


async fn telegram_bot() -> anyhow::Result<(), anyhow::Error> {

    info!("Iniciando bot de telegram...");
    if let Err(e) = forward_notes_to_telegram().await {
        info!("Error: {:?}", e);
    }
    Ok(())
}
