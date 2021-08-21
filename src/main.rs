use tuinance::event::*;

use yahoo_finance::{history, Interval, Streamer, Timestamped};

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

struct Ticker<'a> {
    data: Vec<OrderedFloat<f64>>,
    ticker: &'a str,
}

impl<'a> Ticker<'a> {
    pub fn new(ticker: &'a str) -> Self {
        Self {
            ticker,
            data: vec![],
        }
    }
}

fn next_interval(curr: Interval) -> Interval {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut args = std::env::args();
    args.next();

    let ticker = args.next().unwrap_or("NTDOY".into()).to_uppercase();

    terminal.clear()?;
    let mut size = terminal.size()?;

    let events = Events::new(1000);


    let _streamer = Streamer::new(vec![&ticker.clone()]);
    //let (mut tx, rx) = mpsc::channel();

    /*tokio::spawn(async move {
        streame.stream().await
        .for_each(move |quote| {
            //tx.send(format!("At {}, {} is trading for ${}", quote.timestamp, quote.symbol, quote.price)).unwrap();
            tx.send(quote.price).unwrap();
            future::ready(())
        })
        .await;
    });*/


    let mut chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(100)
        ]).split(size);

    let mut data: Vec<OrderedFloat<f64>> = vec![];

    let p = &OrderedFloat::from(0.0);

    let mut interval = Interval::_6mo;

    let tickers = vec![ticker.clone()];

    let hist = history::retrieve_interval(&ticker, interval).await.unwrap();


    let mut y = vec![];

    for d in hist.iter() {
        data.push(OrderedFloat::from(d.high));
        y.push(format!("{}", d.datetime().format("%b %e %Y")));
    }

    let mut render_list = true;
    let mut is_first_render = true;
    let mut current_index: usize = 0;

    loop {
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
                .name(&ticker)
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

            let span = Span::styled(format!("{}: {}", elem.clone(), data.last().unwrap_or(p)), style);

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
                        format!("TUInance - {} ({})", ticker, interval.to_string()),
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
                .labels([format!("{:.2}", min), format!("{:.2}", max)]
                    .iter()
                    .cloned()
                    .map(|x| Span::styled(x, Style::default().fg(Color::Yellow)))
                    .collect()
                )
            );


        if let Ok(s) = terminal.size() {
            if is_first_render || size != s {
                let constraints = match render_list {
                    true => vec![Constraint::Percentage(15), Constraint::Percentage(85)],
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
            //f.render_widget(Paragraph::new(Text::from(format!("{:#?}", data))).block(Block::default().title("Debug").borders(Borders::ALL)), chunks[0])
        })?;


        /*if let Ok(f) = rx.try_recv() {
            if data.len() > 10 {
                data.remove(0);
            }

            data.push(OrderedFloat::from(f));
        }*/

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
                                        true => vec![Constraint::Percentage(15), Constraint::Percentage(85)],
                                        false => vec![Constraint::Percentage(100)]
                                    };
                                    chunks = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints(constraints)
                                        .split(size);
                                }
                                'l' => {
                                    let int = next_interval(interval);
                                    interval = int;

                                    data.clear();
                                    y.clear();

                                    let hist = history::retrieve_interval(&ticker, interval).await.unwrap();
                                    for d in hist.iter() {
                                        data.push(OrderedFloat::from(d.high));
                                        y.push(format!("{}", d.datetime().format("%b %e %Y")));
                                    }
                                }
                                _ => ()
                            }
                        }
                        _ => ()
                    }
                }
                _ => ()
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
