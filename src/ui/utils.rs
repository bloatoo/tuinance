use tui::layout::{
    Constraint,
    Layout,
    Direction,
    Rect,
};

pub fn generate_chunks(term_size: Rect, render_list: bool) -> (Vec<Rect>, Rect) {
    match render_list {
        true => {
            let constraints = vec![
                Constraint::Percentage(25),
                Constraint::Percentage(75)
            ];
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(term_size);

            let side_panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ]).split(layout[0]);

            (side_panes, layout[1])
        }
        false => {
            let constraints = vec![
                Constraint::Percentage(100)
            ];

            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(term_size);
            (vec![], layout[0])
        }
    }
}
