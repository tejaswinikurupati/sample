use rustls::{Certificate, PrivateKey, ServerConfig};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsAcceptor;
use std::fs::File;
use std::io::BufReader;

fn load_private_key(filename: &str) -> PrivateKey {
    let keyfile = File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .expect("file contains invalid pkcs8 private key");
    PrivateKey(keys.into_iter().next().expect("no private keys found"))
}

fn load_certs(filename: &str) -> Vec<Certificate> {
    let certfile = File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    let certs = rustls_pemfile::certs(&mut reader)
        .expect("file contains invalid certs")
        .into_iter()
        .map(Certificate)
        .collect();
    certs
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let certs = load_certs("ec-cert.pem");
    let key = load_private_key("ec-pkcs8-key.pem");

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let addr = "127.0.0.1:8000";
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            let mut stream = acceptor.accept(stream).await.unwrap();
            let mut buffer = [0; 1024];

            let n = stream.read(&mut buffer).await.unwrap();
            if n == 0 { return; }

            stream.write_all(&buffer[..n]).await.unwrap();
        });
    }
}
