use ordered_float::OrderedFloat;
use crate::ticker::Data;
use yahoo_finance::{Interval, Profile};

pub enum Message {
    DataUpdate((String, Data)),
    ProfileInit((String, Profile)),
    PriceUpdate((String, f64)),
    SetInterval((String, Interval)),
    Start,
}
