use ordered_float::OrderedFloat;
use yahoo_finance::Profile;

pub enum Message {
    DataInit((String, Vec<(OrderedFloat<f64>, String)>)),
    ProfileInit((String, Profile)),
    PriceUpdate((String, f64)),
    IntervalData((String, Vec<(OrderedFloat<f64>, String)>)),
}
