use tuinance::{
    config::Config,
    event::*,
    message::*,
    ticker::Ticker,
    utils::*
};

use yahoo_finance::{Interval, history, Profile, Streamer, Timestamped};
use futures::future;
use std::sync::mpsc::{self, Sender, Receiver};

use ordered_float::OrderedFloat;

use tui::{
    backend::CrosstermBackend,
    layout::{
        Constraint,
        Direction,
        Layout,
    },
    symbols,
    style::{
        Style,
        Modifier,
        Color
    },
    text::Span,
    widgets::{
        Axis,
        Borders,
        Block,
        List,
        ListItem,
        Chart,
        Dataset,
        GraphType,
    },
    Terminal,
};

use crossterm::{
    ExecutableCommand,
    terminal::{
        enable_raw_mode,
        disable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen
    }
};

use std::cmp::Ordering;
use futures::StreamExt;

async fn get_interval_data(symbol: &str, interval: Interval, tx: Sender<Message>) {
    let hist = history::retrieve_interval(symbol, interval).await.unwrap_or(vec![]);

    let mut data = vec![];

    for d in hist.iter() {
        let date = format!("{}", d.datetime().format("%b %e %Y"));
        data.push((OrderedFloat::from(d.close), date));
    }
    tx.send(Message::IntervalData((symbol.to_string(), data))).unwrap();
}

/*async fn init_data(symbol: &str, interval: Interval, tx: Sender<Message>) {
    let hist = history::retrieve_interval(symbol, interval).await.unwrap_or(vec![]);

    let mut data = vec![];

    for d in hist.iter() {
        let date = format!("{}", d.datetime().format("%b %e %Y"));
        data.push((OrderedFloat::from(d.close), date));
    }
    tx.send(Message::DataInit((symbol.to_string(), data))).unwrap();
}*/


async fn get_profile(symbol: &str, tx: Sender<Message>) {
    let profile = Profile::load(symbol).await.unwrap();
    tx.send(Message::ProfileInit((symbol.to_string(), profile))).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut args = std::env::args();
    args.next();

    let user = std::env::var("USER")?;

    let conf = match Config::read(&format!("/home/{}/.config/tuinance.toml", user)) {
        Ok(val) => val,
        Err(_) => Config::default()
    };

    let tickers_str = conf.tickers();

    let (tx, rx) = mpsc::channel::<Message>();

    let mut tickers: Vec<Ticker> = tickers_str.iter().map(|t| {
        Ticker::new(t.to_string())
    }).collect();

    for t in 0..tickers.len() {
        let t = &tickers[t];

        let identifier = t.identifier().clone();
        let identifier_clone = identifier.clone();
        let interval = t.interval().clone();
        let tx_clone = tx.clone();
        let tx_other = tx.clone();

        tokio::spawn(async move {
            get_interval_data(&identifier, interval, tx_clone).await;
        });

        tokio::spawn(async move {
            get_profile(&identifier_clone, tx_other).await;
        });
    }

    terminal.clear()?;
    let mut size = terminal.size()?;

    let events = Events::new(250);

    let streamer = Streamer::new(tickers_str);

    let tx_clone = tx.clone();

    tokio::spawn(async move {
        streamer.stream().await
        .for_each(move |quote| {
            tx_clone.send(Message::PriceUpdate((quote.symbol, quote.price))).unwrap();
            future::ready(())
        })
        .await;
    });

    let mut chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(100)
        ]).split(size);

    let p = &OrderedFloat::from(0.0);

    let mut render_list = true;
    let mut is_first_render = true;
    let mut current_index: usize = 0;
    let mut current_error = String::new();

    loop {
        let ticker = tickers.get(current_index).unwrap();
        let data = ticker.price_data();
        let y = ticker.date_data();

        let len = data.len();
        let floats: Vec<(f64, f64)> = data.iter().enumerate()
            .map(|(idx, &elem)| (idx as f64 + 1.0, f64::from(elem)))
            .collect();

        let min = f64::from(data.iter().min().unwrap_or(&p).clone());
        let max = f64::from(data.iter().max().unwrap_or(&p).clone());

        let f_date = y.first().unwrap_or(&String::new()).to_string();
        let m_date = y.get(y.len() / 2).unwrap_or(&String::new()).to_string();
        let l_date = y.last().unwrap_or(&String::new()).to_string();

        let datasets = vec![
            Dataset::default()
                .name(ticker.identifier())
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&floats),
        ];

        let t: Vec<ListItem> = tickers.iter().enumerate().map(|(idx, elem)| {
            let style = match idx.cmp(&current_index) {
                Ordering::Equal => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                _ => Style::default()
            };

            let name = match elem.info().name().is_empty() {
                true => elem.identifier(),
                false => elem.info().name()
            };

            let span = Span::styled(format!("{}: {:.3}", name, elem.realtime_price()), style);

            ListItem::new(span)
        }).collect();

        let list = List::new(t)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(53, 59, 69)))
            );

        let chart = Chart::new(datasets)
            .block(Block::default()
                   .title(Span::styled(
                        format!("TUInance - {} ({}) {}", ticker.identifier(), ticker.interval().to_string(), current_error),
                        Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD))
                    )
                   .borders(Borders::ALL)
                   .border_style(Style::default().fg(Color::Rgb(53, 59, 69)))
            )
            .style(Style::default().fg(Color::White))
            .x_axis(Axis::default()
                .title(Span::styled(
                    "Date",
                    Style::default().fg(Color::Yellow)
                ))
                .style(Style::default().fg(Color::Rgb(53, 59, 69)))
                .bounds([0.0, len as f64])
                .labels([f_date, m_date, l_date]
                    .iter()
                    .cloned()
                    .map(|x| Span::styled(x, Style::default().fg(Color::Yellow)))
                    .collect()
                )
            )

            .y_axis(Axis::default()
                .title(Span::styled("Price", Style::default().fg(Color::Yellow)))
                .style(Style::default().fg(Color::Rgb(53, 59, 69)))
                .bounds([min, max])
                .labels([format!("{:.3}", min), format!("{:.3}", min + (max - min) / 2.0), format!("{:.3}", max)]
                    .iter()
                    .cloned()
                    .map(|x| Span::styled(x, Style::default().fg(Color::Yellow)))
                    .collect()
                )
            );


        if let Ok(s) = terminal.size() {
            if is_first_render || size != s {
                let constraints = match render_list {
                    true => vec![Constraint::Percentage(20), Constraint::Percentage(80)],
                    false => vec![Constraint::Percentage(100)]
                };
                chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraints)
                    .split(s);

                is_first_render = false;
                size = s;
            }
        }

        terminal.draw(|f| {
            match render_list {
                true => {
                    f.render_widget(chart, chunks[1]);
                    f.render_widget(list, chunks[0]);
                }
                false => {
                    f.render_widget(chart, chunks[0]);
                }
            }
        })?;


        if let Ok(msg) = rx.try_recv() {
            use Message::*;

            match msg {
                DataInit((symbol, data)) => {
                    let ticker = tickers
                        .iter_mut()
                        .find(|t| t.identifier() == &symbol)
                        .unwrap();

                    ticker.init_data(data);
                }

                ProfileInit((symbol, p)) => {
                    let ticker = tickers
                        .iter_mut()
                        .find(|t| t.identifier() == &symbol)
                        .unwrap();

                    ticker.init_info(p);
                }

                IntervalData((symbol, data)) => {
                    let ticker = tickers
                        .iter_mut()
                        .find(|t| t.identifier() == &symbol)
                        .unwrap();

                    ticker.update_data(data);
                }

                PriceUpdate((symbol, price)) => {
                    let ticker = tickers
                        .iter_mut()
                        .find(|t| t.identifier() == &symbol)
                        .unwrap();

                    ticker.set_realtime_price(price);
                }
            }
        }

        if let Ok(ev) = events.next() {
            match ev {
                Event::Input(i) => {
                    match i {
                        Key::Char(c) => {
                            match c {
                                'q' => break,
                                'z' => {
                                    render_list = !render_list;
                                    let constraints = match render_list {
                                        true => vec![Constraint::Percentage(20), Constraint::Percentage(80)],
                                        false => vec![Constraint::Percentage(100)]
                                    };
                                    chunks = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints(constraints)
                                        .split(size);
                                }
                                'j' => {
                                    if current_index + 1 < tickers.len() {
                                        current_index += 1;
                                    }
                                }
                                'k' => {
                                    if current_index >= 1 {
                                        current_index -= 1;
                                    }
                                }
                                'l' => {
                                    let tx = tx.clone();
                                    let ticker = tickers.get_mut(current_index).unwrap();

                                    let next = next_interval(*ticker.interval());

                                    let symbol = ticker.identifier().clone();
                                    let interval = next.clone();

                                    ticker.set_interval(next).await;

                                    tokio::spawn(async move {
                                        get_interval_data(&symbol, interval, tx).await;
                                    });
                                }
                                _ => ()
                            }
                        }
                        _ => ()
                    }
                }
                Event::Tick => (),
            }
        }
    }
    exit()?;

    Ok(())
}

fn exit() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
