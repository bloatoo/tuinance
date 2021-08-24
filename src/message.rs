use ordered_float::OrderedFloat;
use yahoo_finance::{Interval, Profile};

pub enum Message {
    DataInit((String, Vec<(OrderedFloat<f64>, String)>)),
    Start,
    SetInterval((String, Interval)),
    ProfileInit((String, Profile)),
    PriceUpdate((String, f64)),
    IntervalData((String, Vec<(OrderedFloat<f64>, String)>)),
}
