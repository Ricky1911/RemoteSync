use crate::network::{download, upload};

mod network;
#[tokio::main]
async fn main() {
    let result = download(uuid::Uuid::new_v4()).await;
    if let Err(e) = result {
        println!("{:?}", e)
    }
}
