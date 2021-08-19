use tuinance::{
    event::*,
};

use yahoo_finance::Streamer;

use ordered_float::OrderedFloat;

use std::sync::mpsc;

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
        Color
    },
    text::{Text, Span},
    widgets::{
        Axis,
        Borders,
        Block,
        Chart,
        Dataset,
        GraphType,
        Paragraph
    },
    Terminal,
};

use futures::{future, StreamExt};

use crossterm::{
    ExecutableCommand,
    terminal::{
        enable_raw_mode,
        disable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    let size = terminal.size()?;

    let events = Events::new(1000);

    let streamer = Streamer::new(vec!["AAPL"]);
    let (mut tx, rx) = mpsc::channel();

    tokio::spawn(async move {
        streamer.stream().await
        .for_each(move |quote| {
            //tx.send(format!("At {}, {} is trading for ${}", quote.timestamp, quote.symbol, quote.price)).unwrap();
            tx.send(quote.price).unwrap();
            future::ready(())
        })
        .await;
    });


    let mut chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(20),
            Constraint::Percentage(80)
        ]).split(size);

    let mut data: Vec<OrderedFloat<f64>> = vec![];

    loop {
        let floats: Vec<(f64, f64)> = data.iter().enumerate()
            .map(|(idx, &elem)| (idx as f64 + 1.0, f64::from(elem)))
            .collect();

        let p = &OrderedFloat::from(0.0);

        let min = f64::from(data.iter().min().unwrap_or(&p).clone());
        let max = f64::from(data.iter().max().unwrap_or(&p).clone());

        let datasets = vec![
            Dataset::default()
            .name("data1")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&floats),
        ];

        let chart = Chart::new(datasets)
            .block(Block::default().title("Chart").borders(Borders::ALL))
            .x_axis(Axis::default()
                .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 10.0])
                .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()))
            .y_axis(Axis::default()
                .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([min, max])
                .labels([format!("{:.2}", min), format!("{:.2}", max)].iter().cloned().map(Span::from).collect()));

        if let Ok(size) = terminal.size() {
            chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(20),
                    Constraint::Percentage(80)
                ]).split(size);
        }

        terminal.draw(|f| {
            f.render_widget(chart, chunks[1]);
            f.render_widget(Paragraph::new(Text::from(format!("{:#?}", data))).block(Block::default().title("Debug").borders(Borders::ALL)), chunks[0])
        })?;

        if let Ok(f) = rx.try_recv() {
            if data.len() > 10 {
                data.remove(0);
            }

            data.push(OrderedFloat::from(f));
        }

        if let Ok(ev) = events.next() {
            match ev {
                Event::Input(i) => {
                    match i {
                        Key::Char(c) => {
                            match c {
                                'q' => break,
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
