use bevy::app::{AppBuilder, CoreStage, Plugin, PluginGroup, PluginGroupBuilder};
use bevy::asset::{AddAsset, AssetPlugin};
use bevy::core::CorePlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::transform::prelude::TransformPlugin;
use bevy::window::WindowPlugin;

use crossterm::event;

use super::{asset_loaders, components, cursor::Cursor, runner, systems};

pub const PRE_RENDER: &str = "pre_render";
pub const RENDER: &str = "render";
pub const POST_RENDER: &str = "post_render";

#[derive(Default)]
pub struct CrosstermPlugin;
impl Plugin for CrosstermPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Cursor::default())
            .insert_resource(components::PreviousEntityDetails::default())
            .insert_resource(components::EntitiesToRedraw::default())
            .insert_resource(components::PreviousWindowColors::default())
            .add_asset::<components::Sprite>()
            .add_asset::<components::StyleMap>()
            .init_asset_loader::<asset_loaders::SpriteLoader>()
            .init_asset_loader::<asset_loaders::StyleMapLoader>()
            .add_event::<event::KeyEvent>()
            .add_event::<event::MouseEvent>()
            .set_runner(runner::crossterm_runner)
            // Systems and stages
            // This must be before LAST because change tracking is cleared during LAST, but AssetEvents are published
            // after POST_UPDATE. The timing for all these things is pretty delicate
            .add_stage_before(CoreStage::Last, PRE_RENDER, SystemStage::parallel())
            .add_stage_after(PRE_RENDER, RENDER, SystemStage::parallel())
            .add_stage_after(RENDER, POST_RENDER, SystemStage::parallel())
            .add_system_to_stage(CoreStage::PostUpdate, systems::add_previous_position.system())
            // Needs asset events, and they aren't created until after POST_UPDATE, so we put them in PRE_RENDER
            .add_system_to_stage(PRE_RENDER, systems::calculate_entities_to_redraw.system())
            .add_system_to_stage(RENDER, systems::crossterm_render.system())
            .add_system_to_stage(POST_RENDER, systems::update_previous_position.system());
    }
}

pub struct DefaultCrosstermPlugins;

impl PluginGroup for DefaultCrosstermPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        // The crossterm plugin needs many of bevy's plugins, or there
        // will be runtime errors.  Each of the various bevy packages used
        // must include their bevy plugins.  Log and Diagnostics plugins
        // are added primarily just for extra debug information.
        group.add(LogPlugin::default());
        group.add(CorePlugin::default());
        group.add(TransformPlugin::default());
        group.add(DiagnosticsPlugin::default());
        group.add(WindowPlugin::default());
        group.add(AssetPlugin::default());
        // Add crossterm plugin last
        group.add(CrosstermPlugin::default());
    }
}
