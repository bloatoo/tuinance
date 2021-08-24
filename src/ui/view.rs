#[derive(Debug, Clone)]
pub struct View {
    side_panes: Vec<Rect>,
    main_pane: Rect,
}

impl View {
    pub fn focused(main_pane: Rect) -> Self {
        Self {
            side_panes: vec![],
            main_pane
        }
    }

    pub fn normal(list_pane: Rect, info_pane: Rect, main_pane: Rect) {
        Self {
            side_panes: vec![list_pane, info_pane],
            main_pane,
        }
    }
}
