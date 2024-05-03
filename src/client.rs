use rustls::{ClientConfig, RootCertStore, Certificate, ServerName};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;
use std::fs::File;
use std::io::BufReader;

fn load_cert(filename: &str) -> Vec<Certificate> {
    let certfile = File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader).expect("failed to load certificate").into_iter().map(Certificate).collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut root_store = RootCertStore::empty();
    let certs = load_cert("ec-cert.pem");  // Load your server's certificate here
    for cert in certs {
        root_store.add(&cert).unwrap();
    }

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let domain = ServerName::try_from("localhost")?;
    let stream = TcpStream::connect("127.0.0.1:8000").await?;
    let mut stream = connector.connect(domain, stream).await?;

    let msg = b"Successfuly Verified!";
    stream.write_all(msg).await?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}
