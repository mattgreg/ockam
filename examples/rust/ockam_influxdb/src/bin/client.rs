use ockam::{route, Context, Entity, Identity, Result, TcpTransport, Vault, TCP};
use ockam_influxdb::{InfluxClient, InfluxError};
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    // TCP
    let _tcp = TcpTransport::create(&ctx).await?;

    // Secure Channel stuff
    let vault = Vault::create(&ctx)?;

    //    let secure_channel_route = route![(TCP, "localhost:4100"), "secure_channel"];
    //    let secure_channel = client.create_secure_channel(secure_channel_route, TrustEveryonePolicy)?;

    let lease_manager_route = route![(TCP, "127.0.0.1:4000"), "token_lease_service"];
    let mut entity = Entity::create(&ctx, &vault)?;

    // Client application stuff
    let api_url = "http://127.0.0.1:8086";
    let org = "ockam";
    let bucket = "ockam-bucket";
    let leased_token = entity.get_lease(&lease_manager_route, org)?;
    let mut influx_client = InfluxClient::new(api_url, org, bucket, leased_token.value());

    loop {
        let response = influx_client.send_metrics().await;
        if let Err(influx_error) = response {
            if let InfluxError::Authentication(_) = influx_error {
                println!("Authentication failed. Revoking lease.");
                entity.revoke_lease(&lease_manager_route, leased_token.clone())?;
                println!("Press enter to get a new lease");
                let mut tmp = [0_u8; 1];
                std::io::stdin().read(&mut tmp).unwrap();
                let leased_token = entity.get_lease(&lease_manager_route, org)?;
                influx_client.set_token(leased_token.value());
            }
        }
        sleep(Duration::from_secs(1));
    }
    //ctx.stop().await?;
    //Ok(())
}
