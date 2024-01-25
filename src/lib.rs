use eyre::Result;
use hex::decode;
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use k256::ecdsa::{signature::Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc::Receiver;
use tungstenite::{connect, Message};
use url::Url;

const CLIENT_ONE_KEY: &str = "de05b8d4427ecb78b138586181323083fe434df53571bfcbde7a92ec8a96d31f";
const CLIENT_TWO_KEY: &str = "1a8752024402e13781bbc9b5004709d2f673586d54db3a4e61b8c685a1ebe2fd";
const CLIENT_THREE_KEY: &str = "b0c183f8e4ff2e9bd82725d75412aba8106bf45068cb67049b2f217c4806cf68";
const CLIENT_FOUR_KEY: &str = "381a27a834e3342da70b23f72103bbe2bb01d097e1c788a9534ec083e2684e60";
const CLIENT_FIVE_KEY: &str = "4b231ea64938e42acb4e431bca21a1b8125cfb1cfd05c95e0166dc20d7a99b3c";

pub const SIGNING_KEYS: [&str; 5] = [
    CLIENT_ONE_KEY,
    CLIENT_TWO_KEY,
    CLIENT_THREE_KEY,
    CLIENT_FOUR_KEY,
    CLIENT_FIVE_KEY,
];

#[derive(Serialize)]
struct WebSocketRequest {
    id: String,
    method: String,
    params: Params,
}

#[derive(Serialize)]
struct Params {
    symbol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub data_points: Vec<f32>,
    pub average: f32,
}

pub async fn cache(times: usize) -> Result<Data> {
    let (mut socket, _) =
        connect(Url::parse("wss://ws-api.binance.com:443/ws-api/v3")?).expect("can't connect");

    let payload = WebSocketRequest {
        id: "043a7cf2-bde3-4888-9604-c8ac41fcba4d".to_string(),
        method: "ticker.price".to_string(),
        params: Params {
            symbol: "BTCUSDT".to_string(),
        },
    };

    let mut data_points: Vec<f32> = Vec::new();

    for _i in 0..times {
        let serialized = serde_json::to_string(&payload)?;
        socket.send(Message::Text(serialized))?;
        let msg = socket.read().expect("Error reading message");

        data_points.push(
            msg.into_text()?.to_owned()[96..110]
                .parse()
                .expect("should be a valid decimal"),
        );
    }

    let average: f32 = data_points.iter().sum::<f32>() / data_points.len() as f32;

    Ok(Data {
        data_points,
        average,
    })
}

pub fn store_as_json(data: Data) -> Result<()> {
    let json_string = serde_json::to_string(&data).expect("Failed to serialize to JSON");

    let mut file = File::create("output.json").expect("Failed to create file");

    file.write_all(json_string.as_bytes())
        .expect("Failed to write to file");

    Ok(())
}

pub fn read_from_json() -> Option<Data> {
    match std::fs::read_to_string("output.json") {
        Ok(p) => {
            return Some(serde_json::from_str(&p).expect("Error parsing JSON"));
        }
        Err(_) => return None,
    }
}

pub async fn client_process(signing_key: &str) -> Result<(SigningKey, Signature, f32)> {
    let signing_key_bytes = decode(signing_key).expect("Decoding failed");
    let signing_key = SigningKey::from_slice(&signing_key_bytes).expect("Conversion failed");

    let data = cache(10).await?;
    let signature: Signature = signing_key.sign(&data.average.to_be_bytes());

    Ok((signing_key, signature, data.average))
}

pub async fn aggregator_process(
    rx: Receiver<Result<(SigningKey, Signature, f32), eyre::Report>>,
) -> Result<()> {
    let mut averages = Vec::new();

    for _ in 0..5 {
        let (signing_key, sig, avg) = rx.recv()??;

        let verifying_key = VerifyingKey::from(signing_key);
        // dbg!(verifying_key.to_encoded_point(false));
        dbg!(hex::encode(verifying_key.to_sec1_bytes()));
        assert!(verifying_key.verify(&avg.to_be_bytes(), &sig).is_ok());

        averages.push(avg);
    }

    let final_average: f32 = averages.iter().sum::<f32>() / averages.len() as f32;
    println!("{}", final_average);

    Ok(())
}
