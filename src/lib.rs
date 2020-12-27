use bevy::prelude::*;

mod asset_loaders;
pub mod components;
pub mod prelude;
mod runner;
mod systems;

pub struct CrosstermPlugin;
impl Plugin for CrosstermPlugin {
    fn build(&self, app: &mut bevy::app::AppBuilder) {
        app.add_resource(Cursor::default())
            .add_resource(components::PreviousEntityDetails::default())
            .add_resource(components::EntitiesToRedraw::default())
            .add_resource(components::PreviousWindowColors::default())
            .add_asset::<components::Sprite>()
            .add_asset::<components::StyleMap>()
            .init_asset_loader::<asset_loaders::SpriteLoader>()
            .init_asset_loader::<asset_loaders::StyleMapLoader>()
            .add_event::<crossterm::event::KeyEvent>()
            .add_event::<crossterm::event::MouseEvent>()
            .set_runner(runner::crossterm_runner)
            // Systems and stages
            // This must be before LAST because change tracking is cleared during LAST, but AssetEvents are published
            // after POST_UPDATE. The timing for all these things is pretty delicate
            .add_stage_before(
                bevy::app::stage::LAST,
                stage::PRE_RENDER,
                SystemStage::parallel(),
            )
            .add_stage_after(stage::PRE_RENDER, stage::RENDER, SystemStage::parallel())
            .add_stage_after(stage::RENDER, stage::POST_RENDER, SystemStage::parallel())
            .add_system_to_stage(
                bevy::app::stage::POST_UPDATE,
                systems::add_previous_position.system(),
            )
            // Needs asset events, and they aren't created until after POST_UPDATE, so we put them in PRE_RENDER
            .add_system_to_stage(
                stage::PRE_RENDER,
                systems::calculate_entities_to_redraw.system(),
            )
            .add_system_to_stage(stage::RENDER, systems::crossterm_render.system())
            .add_system_to_stage(
                stage::POST_RENDER,
                systems::update_previous_position.system(),
            );
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct WindowSettings {
    colors: components::Colors,
    title: Option<String>,
}

impl Default for WindowSettings {
    fn default() -> Self {
        WindowSettings {
            colors: components::Colors::term_colors(),
            title: None,
        }
    }
}

impl WindowSettings {
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

#[derive(Debug)]
pub struct CrosstermWindow {
    height: u16,
    width: u16,
    colors: components::Colors,
    title: Option<String>,
}

impl Default for CrosstermWindow {
    fn default() -> Self {
        let (width, height) =
            crossterm::terminal::size().expect("Could not read current terminal size");

        let colors = components::Colors::term_colors();
        CrosstermWindow {
            height,
            width,
            colors,
            title: None,
        }
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

#[derive(Debug, Default)]
pub struct Cursor {
    pub x: i32,
    pub y: i32,
    pub hidden: bool,
}

pub mod stage {
    pub const PRE_RENDER: &str = "pre_render";
    pub const RENDER: &str = "render";
    pub const POST_RENDER: &str = "post_render";
}
