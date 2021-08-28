use tuinance::{
    app::{App, GraphType, State},
    config::Config,
    event::*,
    message::*,
    ticker::{Data, Ticker},
    utils::*,
    ui::utils::generate_chunks
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
        Rect,
        Layout,
    },
    symbols,
    style::{
        Style,
        Modifier,
        Color
    },
    text::{Span, Spans},
    widgets::{
        Axis,
        Borders,
        Block,
        List,
        ListItem,
        Chart,
        Dataset,
        GraphType as TuiGraphType,
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
use std::sync::Arc;
use tokio::sync::Mutex;

async fn get_interval_data(symbol: &str, interval: Interval, tx: Sender<Message>) {
    let hist = history::retrieve_interval(symbol, interval).await.unwrap_or(vec![]);

    let mut price_data = vec![];
    let mut date_data = vec![];
    let mut volume_data = vec![];

    for d in hist.iter() {
        let date = format!("{}", d.datetime().format("%b %e %Y"));
        date_data.push(date);
        
        price_data.push(OrderedFloat::from(d.close));
        volume_data.push(d.volume.unwrap());
    }

    let data = Data::new(price_data, date_data, volume_data);
    tx.send(Message::DataUpdate((symbol.to_string(), data))).unwrap();
}

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
    let tx_clone = tx.clone();


    let tickers: Vec<Ticker> = tickers_str.iter().map(|t| {
        Ticker::new(t.to_string())
    }).collect();

    let tickers = Arc::new(Mutex::new(tickers));
    let tickers_clone = tickers.clone();

    std::thread::spawn(move || {
        event_loop(rx, tickers_clone, tx_clone);
    });

    tx.send(Message::Start).unwrap();

    terminal.clear()?;
    let mut size = terminal.size()?;

    let events = Events::new(250);

    let streamer = Streamer::new(tickers_str);

    let mut graph_type = GraphType::Price;

    let tx_clone = tx.clone();

    tokio::spawn(async move {
        streamer.stream().await
        .for_each(move |quote| {
            tx_clone.send(Message::PriceUpdate((quote.symbol, quote.price))).unwrap();
            future::ready(())
        })
        .await;
    });

    let mut chunks: (Vec<Rect>, Rect) = (vec![], Rect::new(0, 0, 0, 0));

    chunks.1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(100)
        ]).split(size)[0];

    let p = &OrderedFloat::from(0.0);

    let mut render_list = true;
    let mut is_first_render = true;
    let mut current_index: usize = 0;
    let current_error = String::new();

    loop {
        let tickers_lock = tickers.lock().await;
        let mut tickers = tickers_lock.clone();
        drop(tickers_lock);

        let ticker = tickers.get(current_index).unwrap();

        let data = match graph_type {
            GraphType::Price => ticker.price_data(),
            GraphType::Volume => ticker.volume_data_f64(),
        };

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
                .graph_type(TuiGraphType::Line)
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

            let span = Span::styled(name, style);

            ListItem::new(span)
        }).collect();

        let list = List::new(t)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(53, 59, 69)))
            );

        let title = match graph_type {
            GraphType::Price => "Price",
            GraphType::Volume => "Volume",
        };

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
                .title(Span::styled(title, Style::default().fg(Color::Yellow)))
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
                chunks = generate_chunks(s, render_list);
                is_first_render = false;
                size = s;
            }
        }

        let info_spans = vec![
            Spans::from(vec![
                Span::styled("Current Price: ", Style::default().fg(Color::Blue)),
                Span::styled(format!("${}", ticker.realtime_price().to_string()), Style::default().fg(Color::Yellow))
            ])
        ];

        let info_list: Vec<ListItem> = info_spans.iter().map(|elem| ListItem::new(elem.clone())).collect();

        let info = List::new(info_list)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(ticker.identifier(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                .border_style(Style::default().fg(Color::Rgb(53, 59, 69)))
            );

        terminal.draw(|f| {
            match render_list {
                true => {
                    f.render_widget(chart, chunks.1);
                    f.render_widget(list, chunks.0[0]);
                    f.render_widget(info, chunks.0[1]);
                }
                false => {
                    f.render_widget(chart, chunks.1);
                }
            }
        })?;

        if let Ok(ev) = events.next() {
            match ev {
                Event::Input(i) => {
                    match i {
                        Key::Char(c) => {
                            match c {
                                'q' => break,
                                'z' => {
                                    render_list = !render_list;
                                    chunks = generate_chunks(size, render_list);
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

                                    tx.send(Message::SetInterval((symbol.clone(), next))).unwrap();
                                    //ticker.set_interval(next).await;

                                    tokio::spawn(async move {
                                        get_interval_data(&symbol, interval, tx).await;
                                    });
                                }
                                'h' => {
                                    let tx = tx.clone();
                                    let ticker = tickers.get_mut(current_index).unwrap();

                                    let prev = previous_interval(*ticker.interval());

                                    let symbol = ticker.identifier().clone();
                                    let interval = prev.clone();

                                    tx.send(Message::SetInterval((symbol.clone(), prev))).unwrap();

                                    tokio::spawn(async move {
                                        get_interval_data(&symbol, interval, tx).await;
                                    });
                                }

                                'v' => {
                                    graph_type = match graph_type {
                                        GraphType::Price => GraphType::Volume,
                                        GraphType::Volume => GraphType::Price
                                    };
                                    let tx = tx.clone();

                                    let ident = ticker.identifier().clone();
                                    let interval = ticker.interval().clone();

                                    tokio::spawn(async move {
                                        get_interval_data(&ident, interval, tx).await;
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

#[tokio::main]
async fn event_loop(rx: Receiver<Message>, tickers: Arc<Mutex<Vec<Ticker>>>, tx: Sender<Message>) {
    loop {
        let msg = match rx.recv() {
            Ok(msg) => msg,
            Err(_) => break,
        };

        let mut tickers = tickers.lock().await;

        use Message::*;

        match msg {
            SetInterval((symbol, interval)) => {
                let ticker = tickers
                    .iter_mut()
                    .find(|t| t.identifier() == &symbol)
                    .unwrap();

                ticker.set_interval(interval);
            }

            ProfileInit((symbol, p)) => {
                let ticker = tickers
                    .iter_mut()
                    .find(|t| t.identifier() == &symbol)
                    .unwrap();

                ticker.init_info(p);
            }

            DataUpdate((symbol, data)) => {
                let ticker = tickers
                    .iter_mut()
                    .find(|t| t.identifier() == &symbol)
                    .unwrap();

                ticker.set_data(data);
            }

            PriceUpdate((symbol, price)) => {
                let ticker = tickers
                    .iter_mut()
                    .find(|t| t.identifier() == &symbol)
                    .unwrap();

                ticker.set_realtime_price(price);
            }
            Start => {
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
            }
        }
    }
}

fn exit() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
