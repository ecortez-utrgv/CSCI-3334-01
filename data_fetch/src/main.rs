use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};

pub trait Pricing {
    fn fetch_price(&mut self) -> Result<(), String>;
    fn save_to_file(&self) -> Result<(), String>;
    fn name(&self) -> &str;
}

#[derive(Debug, Deserialize)]
pub struct Bitcoin {
    pub price: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Ethereum {
    pub price: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct SP500 {
    pub price: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    chart: Chart,
}

#[derive(Debug, Deserialize)]
struct Chart {
    result: Option<Vec<ChartResult>>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct Meta {
    regularMarketPrice: f64,
}

fn fetch_yahoo_price(symbol: &str) -> Result<f64, String> {
    let url = format!(
        "https://query2.finance.yahoo.com/v8/finance/chart/{}",
        symbol
    );

    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("Request error: {}", e))?
        .into_json::<ApiResponse>()
        .map_err(|e| format!("Parse error: {}", e))?;

    response
        .chart
        .result
        .and_then(|mut r| r.pop())
        .map(|r| r.meta.regularMarketPrice)
        .ok_or("Failed to extract price from response".to_string())
}

impl Pricing for Bitcoin {
    fn fetch_price(&mut self) -> Result<(), String> {
        self.price = Some(fetch_yahoo_price("BTC-USD")?);
        Ok(())
    }

    fn save_to_file(&self) -> Result<(), String> {
        let price = self.price.ok_or("No price available")?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("bitcoin.txt")
            .map_err(|e| format!("File error: {}", e))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        writeln!(file, "{} - ${:.2}", timestamp, price).map_err(|e| format!("Write error: {}", e))
    }

    fn name(&self) -> &str {
        "Bitcoin"
    }
}

impl Pricing for Ethereum {
    fn fetch_price(&mut self) -> Result<(), String> {
        self.price = Some(fetch_yahoo_price("ETH-USD")?);
        Ok(())
    }

    fn save_to_file(&self) -> Result<(), String> {
        let price = self.price.ok_or("No price available")?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("ethereum.txt")
            .map_err(|e| format!("File error: {}", e))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        writeln!(file, "{} - ${:.2}", timestamp, price).map_err(|e| format!("Write error: {}", e))
    }

    fn name(&self) -> &str {
        "Ethereum"
    }
}

impl Pricing for SP500 {
    fn fetch_price(&mut self) -> Result<(), String> {
        self.price = Some(fetch_yahoo_price("%5EGSPC")?);
        Ok(())
    }

    fn save_to_file(&self) -> Result<(), String> {
        let price = self.price.ok_or("No price available")?;
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("sp500.txt")
            .map_err(|e| format!("File error: {}", e))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        writeln!(file, "{} - ${:.2}", timestamp, price).map_err(|e| format!("Write error: {}", e))
    }

    fn name(&self) -> &str {
        "S&P 500"
    }
}

fn main() {
    let delay = time::Duration::from_secs(10);

    let mut assets: Vec<Box<dyn Pricing>> = vec![
        Box::new(Bitcoin { price: None }),
        Box::new(Ethereum { price: None }),
        Box::new(SP500 { price: None }),
    ];

    loop {
        for asset in assets.iter_mut() {
            println!("Fetching price for {}", asset.name());

            match asset.fetch_price() {
                Ok(_) => println!("Fetched successfully."),
                Err(e) => println!("Error fetching {}: {}", asset.name(), e),
            }

            match asset.save_to_file() {
                Ok(_) => println!("Saved successfully.\n"),
                Err(e) => println!("Error saving {}: {}\n", asset.name(), e),
            }
        }

        thread::sleep(delay);
    }
}
