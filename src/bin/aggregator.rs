use crypto_price_aggregator::{aggregator_process, client_process, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut handles = Vec::new();

    let (tx, rx) = ::std::sync::mpsc::channel();

    for i in 0..5 {
        let tx = tx.clone();
        handles.push(tokio::spawn(
            async move { tx.send(client_process(i + 1).await) },
        ));
    }

    let aggregator_handle = tokio::spawn(async move {
        aggregator_process(rx).await.unwrap();
    });

    for handle in handles {
        handle.await??;
    }

    aggregator_handle.await?;

    Ok(())
}
