use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    tickers: Vec<String>
}

impl Config {
    pub fn default() -> Self {
        Self {
            tickers: vec!["MSFT".into()]
        }
    }

    pub fn tickers(&self) -> Vec<&str> {
        self.tickers.iter().map(|elem| elem.as_str()).collect::<Vec<&str>>().clone()
    }
    
    pub fn read(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;

        match toml::from_str(&contents) {
            Ok(val) => Ok(val),
            Err(_) => Ok(Self::default())
        }
    }
}

impl<T> From<T> for Config 
where
    T: Into<String>
{
    fn from(data: T) -> Self {
        match toml::from_str(&data.into()) {
            Ok(val) => val,
            Err(_) => Self::default()
        }
    }
}
