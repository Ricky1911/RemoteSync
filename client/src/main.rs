use crate::network::upload;

mod network;
#[tokio::main]
async fn main() {
    upload("Cargo.toml").await
}
