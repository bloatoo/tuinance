use tuinance::{
    event::*,
};

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
    text::Span,
    widgets::{
        Axis,
        Block,
        Chart,
        Dataset,
        GraphType
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

    let mut chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(20),
            Constraint::Percentage(80)
        ]).split(size);

    loop {
        let datasets = vec![
            Dataset::default()
            .name("data1")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Yellow))
                .data(&[(0.0, 42000.0), (1.0, 47000.0), (2.0, 43000.0), (3.0, 49000.0), (4.0, 48000.0), (5.0, 47500.0), (6.0, 43000.0)]),
        ];

        let chart = Chart::new(datasets)
            .block(Block::default().title("Chart"))
            .x_axis(Axis::default()
                .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, 10.0])
                .labels(["0.0", "5.0", "10.0"].iter().cloned().map(Span::from).collect()))
            .y_axis(Axis::default()
                .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                .style(Style::default().fg(Color::White))
                .bounds([40000.0, 50000.0])
                .labels(["40000", "42500", "45000", "47500", "50000"].iter().cloned().map(Span::from).collect()));

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
        })?;

        if let Ok(Event::Input(ev)) = events.next() {
            match ev {
                Key::Char(c) => {
                    match c {
                        'q' => break,
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
