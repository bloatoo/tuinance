use ordered_float::OrderedFloat;
use yahoo_finance::{Profile, history, Interval, Timestamped};
#[derive(Clone, Debug)]
pub struct Info {
    name: String,
}

impl From<Profile> for Info {
    fn from(p: Profile) -> Self {
        let name = match p {
            Profile::Company(p) => p.name,
            Profile::Fund(p) => p.name,
        };

        Self {
            name
        }
    }
}

impl Info {
    pub fn unknown() -> Self {
        Self {
            name: String::new(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct Data {
    price_data: Vec<OrderedFloat<f64>>,
    date_data: Vec<String>,
    volume_data: Vec<u64>,
}

impl Data {
    pub fn empty() -> Self {
        Self {
            price_data: vec![],
            date_data: vec![],
            volume_data: vec![],
        }
    }

    pub fn new(price_data: Vec<OrderedFloat<f64>>, date_data: Vec<String>, volume_data: Vec<u64>) -> Self {
        Self {
            price_data,
            date_data,
            volume_data,
        }
    }

    pub fn price_data_mut(&mut self) -> &mut Vec<OrderedFloat<f64>> {
        &mut self.price_data
    }

    pub fn price_data(&self) -> &Vec<OrderedFloat<f64>> {
        &self.price_data
    }

    pub fn volume_data(&self) -> &Vec<u64> {
        &self.volume_data
    }

    pub fn set_price_data(&mut self, data: Vec<OrderedFloat<f64>>) {
        self.price_data = data;
    }
}

#[derive(Clone, Debug)]
pub struct Ticker {
    data: Data,
    info: Info,
    interval: Interval,
    identifier: String,
    realtime_price: f64,
}

impl Ticker {
    pub fn new(identifier: String) -> Ticker {
        Self {
            identifier,
            interval: Interval::_6mo,
            realtime_price: 0.0,
            info: Info::unknown(),
            data: Data::empty(),
        }
    }

    pub fn identifier(&self) -> &String {
        &self.identifier
    }

    pub fn interval(&self) -> &Interval {
        &self.interval
    }

    pub fn set_realtime_price(&mut self, val: f64) {
        self.realtime_price = val;
    }

    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn volume_data(&self) -> &Vec<u64> {
        &self.data.volume_data()
    }

    pub fn volume_data_f64(&self) -> Vec<OrderedFloat<f64>> {
        self.data.volume_data()
            .iter()
            .map(|elem| OrderedFloat::from(*elem as f64))
            .collect()
    }

    pub fn realtime_price(&self) -> f64 {
        if self.realtime_price == 0.0 {
            let placeholder = &OrderedFloat::from(0.0);
            f64::from(self.price_data().iter().last().unwrap_or(placeholder).clone())
        } else {
            self.realtime_price
        }
    }

    pub fn set_data(&mut self, data: Data) {
        self.data = data;
    }

    pub fn set_interval(&mut self, interval: Interval) {
        self.interval = interval;
    }

    pub async fn get_profile(&mut self) {
        let profile = Profile::load(&self.identifier).await.unwrap();
        self.info = Info::from(profile);
    }

    pub fn init_info(&mut self, profile: Profile) {
        self.info = Info::from(profile);
    }

    pub async fn get_data(&mut self) -> Result<(), yahoo_finance::Error> {
        let hist = history::retrieve_interval(&self.identifier, self.interval).await?;

        let mut price_data = vec![];
        let mut date_data = vec![];
        let mut volume_data = vec![];

        for d in hist.iter() {
            price_data.push(OrderedFloat::from(d.close));
            date_data.push(format!("{}", d.datetime().format("%b %e %Y")));
            volume_data.push(d.volume.unwrap());
        }

        self.data = Data::new(price_data, date_data, volume_data);
        
        Ok(())
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    pub fn price_data(&self) -> Vec<OrderedFloat<f64>> {
        self.data.price_data().clone()
    }

    pub fn date_data(&self) -> &Vec<String> {
        &self.data.date_data
    }
}
