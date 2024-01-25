use oracle_takehome::{aggregator_process, client_process, SIGNING_KEYS};

#[tokio::main]
async fn main() {
    let mut handles = Vec::new();

    // Create a simple streaming channel
    let (tx, rx) = ::std::sync::mpsc::channel();

    for i in 0..5 {
        let tx = tx.clone();
        handles.push(tokio::spawn(async move {
            tx.send(client_process(SIGNING_KEYS[i]).await)
        }));
    }

    let aggregator_handle = tokio::spawn(async move {
        aggregator_process(rx).await.unwrap();
    });

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    aggregator_handle.await.unwrap();
}
