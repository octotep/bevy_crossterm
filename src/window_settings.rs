use super::components;

#[derive(Clone, Eq, PartialEq)]
pub struct CrosstermWindowSettings {
    colors: components::Colors,
    title: Option<String>,
}

impl Default for CrosstermWindowSettings {
    fn default() -> Self {
        CrosstermWindowSettings {
            colors: components::Colors::term_colors(),
            title: None,
        }
    }
}

impl CrosstermWindowSettings {
    pub fn colors(&self) -> components::Colors {
        self.colors
    }

    pub fn title(&self) -> &Option<String> {
        &self.title
    }

    pub fn set_title<T: std::string::ToString>(&mut self, title: T) -> &mut Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn set_colors(&mut self, colors: components::Colors) -> &mut Self {
        self.colors = colors;
        self
    }
}
