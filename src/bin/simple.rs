use oracle_takehome::{cache, read_from_json, store_as_json, Parser, Result};

/// A basic CLi for calculating the average price of BTCUSDT
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mode to run. Cache or Read.
    #[arg(short, long)]
    mode: String,

    /// Number of times to listen to websocket
    #[arg(short, long, default_value_t = 1)]
    times: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mode = args.mode.as_str();
    match mode {
        "cache" => {
            let data = cache(args.times).await?;

            println!(
                "Cache complete.\nThe average USD price of BTC is: {}",
                data.average
            );

            store_as_json(data)?;
        }
        "read" => {
            let data = read_from_json().expect("should have data");
            println!(
                "Data points: {:?} \nAverage: {}",
                data.data_points, data.average
            );
        }
        _ => println!("Unknown mode: {}. Valid modes are `cache` and `read`", mode),
    }

    Ok(())
}
