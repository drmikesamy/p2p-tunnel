#![allow(deprecated)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use iroh::{Endpoint, EndpointAddr, SecretKey, endpoint::presets};
use iroh_tickets::endpoint::EndpointTicket;
use tokio::io::copy;
use tokio::net::{TcpListener, TcpStream};

const ALPN: &[u8] = b"p2p-tunnel/v1";

#[derive(Parser)]
#[command(name = "p2p-tunnel", about = "Instantly share localhost ports")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Share a local port (e.g., `p2p-tunnel share 3000`)
    Share { port: u16 },
    /// Connect to a shared port (e.g., `p2p-tunnel connect <TICKET>`)
    Connect { ticket: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Share { port } => share(port).await?,
        Commands::Connect { ticket } => connect(ticket).await?,
    }

    Ok(())
}

async fn share(port: u16) -> Result<()> {
    // 1. Initialize an Iroh Endpoint
    let secret_key = SecretKey::generate();
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(secret_key)
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await?;

    // Wait until we've discovered connectivity info before creating the ticket.
    endpoint.online().await;

    // 2. Generate a connection ticket
    let addr = endpoint.addr();
    let ticket = EndpointTicket::from(addr).to_string();

    println!("🚀 Sharing localhost:{}", port);
    println!("📋 Give this command to your friend:\n");
    println!("    npx p2p-tunnel connect {}\n", ticket);
    println!("Waiting for connections...");

    // 3. Listen for incoming Iroh P2P connections
    while let Some(incoming) = endpoint.accept().await {
        let connection = incoming.await?;
        println!("🔗 Peer connected!");

        // 4. Accept a bidirectional stream from the peer
        let (mut iroh_send, mut iroh_recv) = connection.accept_bi().await?;

        // 5. Open a local TCP connection to the service you are sharing
        tokio::spawn(async move {
            let mut local_tcp = TcpStream::connect(format!("127.0.0.1:{}", port))
                .await
                .expect("Failed to connect to local service");

            // 6. Shovel bytes back and forth
            let (mut local_read, mut local_write) = local_tcp.split();
            let local_to_remote = async {
                copy(&mut local_read, &mut iroh_send).await?;
                iroh_send.finish()?;
                Result::<()>::Ok(())
            };
            let remote_to_local = async {
                copy(&mut iroh_recv, &mut local_write).await?;
                Result::<()>::Ok(())
            };

            let _ = tokio::try_join!(local_to_remote, remote_to_local);
        });
    }
    Ok(())
}

async fn connect(ticket: String) -> Result<()> {
    // 1. Parse the ticket back into an EndpointAddr
    let ticket: EndpointTicket = ticket.parse()?;
    let host_addr: EndpointAddr = ticket.into();

    // 2. Initialize a local Iroh Endpoint
    let endpoint = Endpoint::bind(presets::N0).await?;

    // 3. Start a local TCP listener to act as the proxy
    let local_listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("🌍 Connected! Service available at: http://localhost:8080");

    loop {
        // 4. Wait for the user's browser to hit localhost:8080
        let (mut local_tcp, _) = local_listener.accept().await?;

        let endpoint = endpoint.clone();
        let host_addr = host_addr.clone();

        tokio::spawn(async move {
            // 5. Dial the remote Host over Iroh
            let connection = endpoint.connect(host_addr, ALPN).await.unwrap();
            let (mut iroh_send, mut iroh_recv) = connection.open_bi().await.unwrap();

            // 6. Shovel bytes
            let (mut local_read, mut local_write) = local_tcp.split();
            let local_to_remote = async {
                copy(&mut local_read, &mut iroh_send).await?;
                iroh_send.finish()?;
                Result::<()>::Ok(())
            };
            let remote_to_local = async {
                copy(&mut iroh_recv, &mut local_write).await?;
                Result::<()>::Ok(())
            };

            let _ = tokio::try_join!(local_to_remote, remote_to_local);
        });
    }
}