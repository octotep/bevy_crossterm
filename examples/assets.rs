use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use std::default::Default;

#[derive(Clone)]
enum GameState {
    Loading,
    Running,
}

static STAGE: &str = "GAME";

// PROTIP: _technically_ since Sprite's are just created using strings, an easier way to load them from an external
// file is just:
//static TITLE_TEXT: &str = include_str!("assets/demo/title.txt");
// then just:
//sprites.add(Sprite::new(TITLE_TEXT));
// and boom, you have yourself a sprite in the asset system.
// That's nice and easy - don't have to worry about async, don't need to distribute files alongside your exe.
// But then you can't take advangate of hot reloading, and plus it only works for sprites. StyleMaps have to go through
// the AssetServer if you want to load them from an external file.

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("Assets example");

    App::build()
        // Add our window settings
        .add_resource(settings)
        // Set some options in bevy to make our program a little less resource intensive - it's just a terminal game
        // no need to try and go nuts
        .add_resource(bevy::core::DefaultTaskPoolOptions::with_num_threads(1))
        // The Crossterm runner respects the schedulerunnersettings. No need to run as fast as humanly
        // possible - 20 fps should be more than enough for a scene that never changes
        .add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
            std::time::Duration::from_millis(50),
        ))
        .add_stage_after(stage::UPDATE, STAGE, StateStage::<GameState>::default())
        .add_resource(State::new(GameState::Loading))
        .on_state_enter(STAGE, GameState::Loading, loading_system.system())
        .on_state_update(STAGE, GameState::Loading, check_for_loaded.system())
        .on_state_enter(STAGE, GameState::Running, create_entities.system())
        .add_plugins(DefaultPlugins)
        .add_plugin(CrosstermPlugin)
        .run();
}

static ASSETS: &[&str] = &["demo/title.txt", "demo/title.stylemap"];

// This is a simple system that loads assets from the filesystem
fn loading_system(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut cursor: ResMut<Cursor>,
) {
    cursor.hidden = true;

    // Load the assets we want
    let mut handles = Vec::new();
    for asset in ASSETS {
        handles.push(asset_server.load_untyped(*asset));
    }

    commands.insert_resource(handles);
}

// This function exists soely because bevy's asset loading is async.
// We need to wait until all assets are loaded before we do anyhting with them.
fn check_for_loaded(
    asset_server: Res<AssetServer>,
    handles: Res<Vec<HandleUntyped>>,
    mut state: ResMut<State<GameState>>,
) {
    let data = asset_server.get_group_load_state(handles.iter().map(|handle| handle.id));

    match data {
        bevy::asset::LoadState::NotLoaded | bevy::asset::LoadState::Loading => {}
        bevy::asset::LoadState::Loaded => {
            state.set_next(GameState::Running).unwrap();
        }
        bevy::asset::LoadState::Failed => {}
    }
}

// Now that we're sure the assets are loaded, spawn a new sprite into the world
fn create_entities(
    commands: &mut Commands,
    window: Res<CrosstermWindow>,
    asset_server: Res<AssetServer>,
    sprites: Res<Assets<Sprite>>,
) {
    // I want to center the title, so i needed to wait until it was loaded before I could actually access
    // the underlying data to see how wide the sprite is and do the math
    let title_handle = asset_server.get_handle("demo/title.txt");
    let title_sprite = sprites.get(&title_handle).unwrap();

    let center_x = window.x_center() as i32 - title_sprite.x_center() as i32;
    let center_y = window.y_center() as i32 - title_sprite.y_center() as i32;

    commands.spawn(SpriteBundle {
        sprite: title_handle.clone(),
        position: Position::with_xy(center_x, center_y),
        stylemap: asset_server.get_handle("demo/title.stylemap"),
        ..Default::default()
    });
}
