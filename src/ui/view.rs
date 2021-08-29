use tui::layout::Rect;

#[derive(Debug, Clone)]
pub struct View {
    info_pane: Rect,
    list_pane: Rect,
    volume_pane: Rect,
    main_pane: Rect,
}

impl View {
    pub fn normal(list_pane: Rect, info_pane: Rect, main_pane: Rect, volume_pane: Rect) -> Self {
        Self {
            info_pane,
            list_pane,
            volume_pane,
            main_pane
        }
    }
}
