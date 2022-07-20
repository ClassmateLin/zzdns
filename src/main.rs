use anyhow::Result;
use zzdns::init_servers;




#[tokio::main]
async fn main() -> Result<()>{
    pretty_env_logger::init_timed();
    init_servers().await
}