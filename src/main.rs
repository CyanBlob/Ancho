use iced::{Application, Settings};

mod app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = app::HomePage::run(Settings::default());
    match res {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }
}
