use super::components;

#[derive(Debug)]
pub struct CrosstermWindow {
    pub(crate) height: u16,
    pub(crate) width: u16,
    pub(crate) colors: components::Colors,
    pub(crate) title: Option<String>,
}

impl Default for CrosstermWindow {
    fn default() -> Self {
        let (width, height) = crossterm::terminal::size().expect("Could not read current terminal size");

        let colors = components::Colors::term_colors();
        CrosstermWindow { height, width, colors, title: None }
    }
}

impl CrosstermWindow {
    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn colors(&self) -> components::Colors {
        self.colors
    }

    pub fn set_colors(&mut self, new_colors: components::Colors) {
        self.colors = new_colors;
    }

    pub fn x_center(&self) -> u16 {
        self.width / 2
    }

    pub fn y_center(&self) -> u16 {
        self.height / 2
    }
}
