#[derive(Debug, Clone)]
pub enum GraphType {
    Price,
    Volume
}


pub enum State {
    Main,
    ModeSelection,
}

pub struct App {
    state: State,
    graph_type: GraphType,
}
