#[derive(Default)]
pub struct AppState {
    pub current_screen: Screen,
}

#[derive(Default, Clone)]
pub enum Screen {
    #[default]
    Home,
    Session(String),
    Dialog(Box<DialogState>),
}

#[derive(Clone)]
pub enum DialogState {
    Alert {
        message: String,
    },
    Confirm {
        message: String,
    },
    Prompt {
        message: String,
        input: String,
    },
    Provider,
    Agent,
    ProvidersList,
    Help,
    /// Command list (slash); carries session_id to restore when closing.
    Command(String),
}
