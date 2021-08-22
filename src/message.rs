use ordered_float::OrderedFloat;

pub struct TickerData {
    symbol: String,
    data: Vec<(OrderedFloat<f64>, String)>,
}

pub enum Message {
    Init(TickerData),
    PriceUpdate((f64, String)),
    IntervalData((Vec<(OrderedFloat<f64>, String)>, String)),
}
