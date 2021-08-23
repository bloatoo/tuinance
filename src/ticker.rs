use ordered_float::OrderedFloat;
use yahoo_finance::{Profile, history, Interval, Timestamped};

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

pub struct Data {
    price_data: Vec<OrderedFloat<f64>>,
    date_data: Vec<String>,
    volume_data: Vec<f64>,
}

impl Data {
    pub fn empty() -> Self {
        Self {
            price_data: vec![],
            date_data: vec![],
            volume_data: vec![],
        }
    }
}

pub struct Ticker {
    data: Vec<(OrderedFloat<f64>, String)>,
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
            data: vec![],
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

    pub fn realtime_price(&self) -> f64 {
        if self.realtime_price == 0.0 {
            let placeholder = &OrderedFloat::from(0.0);
            f64::from(self.price_data().iter().last().unwrap_or(placeholder).clone())
        } else {
            self.realtime_price
        }
    }

    pub async fn set_interval(&mut self, interval: Interval) {
        self.interval = interval;
    }

    pub async fn get_profile(&mut self) {
        let profile = Profile::load(&self.identifier).await.unwrap();
        self.info = Info::from(profile);
    }

    pub fn init_data(&mut self, data: Vec<(OrderedFloat<f64>, String)>) {
        self.data = data;
        self.realtime_price = f64::from(*self.data.last().unwrap().0);
    }

    pub fn update_data(&mut self, data: Vec<(OrderedFloat<f64>, String)>) {
        self.data = data;
    }
    
    pub fn init_info(&mut self, profile: Profile) {
        self.info = Info::from(profile);
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
