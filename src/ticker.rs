use ordered_float::OrderedFloat;
use yahoo_finance::{Profile, history, Interval, Timestamped};
use crate::utils::interval_to_days;

pub struct Ticker<'a> {
    data: Vec<(OrderedFloat<f64>, String)>,
    max_data: Vec<(OrderedFloat<f64>, String)>,
    name: String,
    interval: Interval,
    identifier: &'a str,
    realtime_price: f64,
}

impl<'a> Ticker<'a> {
    pub fn new(identifier: &'a str) -> Ticker<'a> {
        Self {
            identifier,
            interval: Interval::_6mo,
            realtime_price: 0.0,
            name: String::new(),
            data: vec![],
            max_data: vec![],
        }
    }

    pub fn identifier(&self) -> &'a str {
        &self.identifier
    }

    pub fn interval(&self) -> &Interval {
        &self.interval
    }

    pub fn max_data(&self) -> &Vec<(OrderedFloat<f64>, String)> {
        &self.max_data
    }

    pub fn set_realtime_price(&mut self, val: f64) {
        self.realtime_price = val;
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn realtime_price(&self) -> f64 {
        let placeholder = &OrderedFloat::from(0.0);
        f64::from(self.price_data().iter().last().unwrap_or(placeholder).clone())
    }

    pub async fn set_interval(&mut self, interval: Interval) -> Result<(), yahoo_finance::Error> {
        self.interval = interval;
        self.get_data().await

        /*let days = interval_to_days(interval) as usize;
        let max_len = self.max_data.len();

        if days > max_len || days == 0 {
            self.data = self.max_data.clone();
            return;
        }

        self.data = self.max_data[max_len - days..max_len].to_vec();*/
    }

    pub async fn get_profile(&mut self) {
        let profile = Profile::load(&self.identifier).await.unwrap();

        self.name = match profile {
            Profile::Company(c) => {
                c.name
            }
            Profile::Fund(f) => {
                f.name
            }
        };
    }

    pub async fn init(&mut self) -> Result<(), yahoo_finance::Error> {
        self.get_data().await?;
        self.get_profile().await;

        self.realtime_price = f64::from(*self.data.last().unwrap().0);
        Ok(())
    }

    pub async fn get_data(&mut self) -> Result<(), yahoo_finance::Error> {
        let hist = history::retrieve_interval(&self.identifier, self.interval).await?;

        let mut data = vec![];

        for d in hist.iter() {
            let date = format!("{}", d.datetime().format("%b %e %Y"));
            data.push((OrderedFloat::from(d.close), date));
        }

        self.data = data.clone();
        Ok(())

        /*self.max_data = data;

        let days = interval_to_days(self.interval) as usize;

        let max_len = self.max_data.len();

        if days > max_len || days == 0 {
            self.data = self.max_data.clone();
            return;
        }

        self.data = self.max_data[max_len - days..max_len].to_vec();*/
    }

    pub fn data(&self) -> &Vec<(OrderedFloat<f64>, String)> {
        &self.data
    }

    pub fn price_data(&self) -> Vec<OrderedFloat<f64>> {
        self.data.iter().map(|elem| elem.0).collect()
    }

    pub fn date_data(&self) -> Vec<String> {
        self.data.iter().map(|elem| elem.1.clone()).collect()
    }
}
