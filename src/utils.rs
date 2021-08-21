use yahoo_finance::Interval;

pub fn next_interval(curr: Interval) -> Interval {
    use Interval::*;
    match curr {
        _1mo => _3mo,
        _3mo => _6mo,
        _6mo => _1y,
        _1y => _2y,
        _2y => _5y,
        _5y => _10y,
        _10y => _max,
        _max => _1mo,
        _ => _1mo
    }
}

pub fn interval_to_days(int: Interval) -> u32 {
    use Interval::*;
    match int {
        _1mo => 30,
        _3mo => 91,
        _6mo => 183,
        _1y => 365,
        _2y => 730,
        _5y => 1825,
        _10y => 3650,
        _ => 0,
    }
}
