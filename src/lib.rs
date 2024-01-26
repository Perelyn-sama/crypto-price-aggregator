pub use eyre::Result;
use hex::decode;
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use k256::ecdsa::{signature::Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc::Receiver;
use tungstenite::{connect, Message};
use url::Url;

const CLIENT_ONE_PUBLICKEY: &str =
    "02a442d13ba89bbaeb766b1e145e3ea07db831e1a4f418fad82d8323bf2ca8218e";
const CLIENT_TWO_PUBLICKEY: &str =
    "02ab02d2d0589dd072a5323f6bab8ff8690664b1f8a7f6e8a339b9902ad73cfe27";
const CLIENT_THREE_PUBLICKEY: &str =
    "02ad8b749821280e748a1de7114aad18bb4e10750815270c0d9d008af99238f47f";
const CLIENT_FOUR_PUBLICKEY: &str =
    "0355aeda8cfabf9abf206e6db057989534a28ba3338769d5473834844f147dc603";
const CLIENT_FIVE_PUBLICKEY: &str =
    "031cb5014532b69a8521c7642bde88b47bff0eada8b1324e49a8b8a65670e987ca";

pub const VERIFYING_KEYS: [&str; 5] = [
    CLIENT_ONE_PUBLICKEY,
    CLIENT_TWO_PUBLICKEY,
    CLIENT_THREE_PUBLICKEY,
    CLIENT_FOUR_PUBLICKEY,
    CLIENT_FIVE_PUBLICKEY,
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

pub async fn client_process(client_id: u32) -> Result<(u32, Signature, f32)> {
    // Load variables from .env file
    dotenv::dotenv().ok();

    let env_key = format!("CLIENTSIGNINGKEY{}", client_id);
    let client_signing_key = env::var(env_key).expect("Key not found in environment");

    let signing_key_bytes = decode(client_signing_key).expect("Decoding failed");
    let signing_key = SigningKey::from_slice(&signing_key_bytes).expect("Conversion failed");

    let data = cache(10).await?;
    let signature: Signature = signing_key.sign(&data.average.to_be_bytes());

    Ok((client_id, signature, data.average))
}

pub async fn aggregator_process(
    rx: Receiver<Result<(u32, Signature, f32), eyre::Report>>,
) -> Result<()> {
    let mut averages = Vec::new();

    for _ in 0..5 {
        let (client_id, sig, avg) = rx.recv()??;

        let verifying_key =
            VerifyingKey::from_sec1_bytes(&decode(VERIFYING_KEYS[client_id as usize - 1])?)?;

        assert!(verifying_key.verify(&avg.to_be_bytes(), &sig).is_ok());

        averages.push(avg);
    }

    let final_average: f32 = averages.iter().sum::<f32>() / averages.len() as f32;
    println!("{}", final_average);

    Ok(())
}
