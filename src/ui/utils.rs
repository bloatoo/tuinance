use tui::layout::{
    Constraint,
    Layout,
    Direction,
    Rect,
};

pub fn generate_chunks(term_size: Rect, render_list: bool) -> (Vec<Rect>, Vec<Rect>) {
    match render_list {
        true => {
            let constraints = vec![
                Constraint::Percentage(20),
                Constraint::Percentage(80)
            ];
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints.clone())
                .split(term_size);

            let main_panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ])
                .split(layout[1]);

            let side_panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ]).split(layout[0]);

            (side_panes, main_panes)
        }
        false => {
            let constraints = vec![
                Constraint::Percentage(100)
            ];

            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(term_size);

            (vec![], layout)
        }
    }
}
