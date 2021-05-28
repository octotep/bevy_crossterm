use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use std::default::Default;

mod animation;
mod colors;
mod finale;
mod sprites;
mod title;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Loading = 0,
    Title,
    Sprites,
    Colors,
    Animation,
    Finale,
}

impl GameState {
    pub fn next_state(&self) -> Option<GameState> {
        use GameState::*;
        match self {
            Loading => Some(Title),
            Title => Some(Sprites),
            Sprites => Some(Colors),
            Colors => Some(Animation),
            Animation => Some(Finale),
            Finale => None,
        }
    }
}

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("bevy_crossterm demo");

    App::build()
        .insert_resource(settings)
        .insert_resource(bevy::app::ScheduleRunnerSettings::run_loop(
            std::time::Duration::from_millis(16),
        ))
        .add_plugins(DefaultPlugins)
        .add_plugin(CrosstermPlugin)
        .add_startup_system(demo_setup.system())
        .add_startup_system(loading_system.system())
        .add_state(GameState::Loading)
        .add_system_set(SystemSet::on_update(GameState::Loading).with_system(check_for_loaded.system()))

        .add_system_set(SystemSet::on_enter(GameState::Title).with_system(title::setup.system()))
        .add_system_set(SystemSet::on_update(GameState::Title).with_system(just_wait_and_advance.system()))
        .add_system_set(SystemSet::on_exit(GameState::Title).with_system(simple_teardown.system()))
        
        .add_system_set(SystemSet::on_enter(GameState::Sprites).with_system(sprites::setup.system()))
        .add_system_set(SystemSet::on_update(GameState::Sprites).with_system(just_wait_and_advance.system()))
        .add_system_set(SystemSet::on_exit(GameState::Sprites).with_system(simple_teardown.system()))

        .add_system_set(SystemSet::on_enter(GameState::Colors).with_system(colors::setup.system()))
        .add_system_set(SystemSet::on_update(GameState::Colors).with_system(just_wait_and_advance.system()))
        .add_system_set(SystemSet::on_exit(GameState::Colors).with_system(simple_teardown.system()))

        .add_system_set(SystemSet::on_enter(GameState::Animation).with_system(animation::setup.system()))
        .add_system_set(SystemSet::on_update(GameState::Animation).with_system(animation::update.system()))
        .add_system_set(SystemSet::on_exit(GameState::Animation).with_system(simple_teardown.system()))

        .add_system_set(SystemSet::on_enter(GameState::Finale).with_system(finale::setup.system()))
        .add_system_set(SystemSet::on_update(GameState::Finale).with_system(just_wait_and_advance.system()))
        .add_system_set(SystemSet::on_exit(GameState::Finale).with_system(simple_teardown.system()))
        
        .run();
}

fn loading_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut cursor: ResMut<Cursor>,
) {
    cursor.hidden = true;

    // Load the assets we want
    let handles = asset_server.load_folder("demo").unwrap();

    asset_server.watch_for_changes().unwrap();

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
            let next_state = state.current().next_state().unwrap();
            state.push(next_state).unwrap();
        }
        bevy::asset::LoadState::Failed => {}
    }
}

// Setup anything needed globally for the demo
pub fn demo_setup(mut commands: Commands) {
    let scene_root = commands.spawn_bundle(()).id();

    commands.insert_resource(scene_root);
}

// Helper function to see if there was a key press this frame
pub fn detect_keypress(keys: &mut EventReader<KeyEvent>) -> bool {
    keys.iter().last().is_some()
}

// Simple update function that most screens will use
pub fn just_wait_and_advance(
    mut state: ResMut<State<GameState>>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
    mut keys: EventReader<KeyEvent>,
) {
    if detect_keypress(&mut keys) {
        if let Some(next_stage) = state.current().next_state() {
            state.push(next_stage).unwrap();
        } else {
            app_exit.send(bevy::app::AppExit);
        }
    }
}

// Looks for an entity resource and then despawns that entity and all it's children
pub fn simple_teardown(mut commands: Commands, mut scene_root: ResMut<Entity>) {
    commands.entity(*scene_root).despawn_recursive();

    // Create a new, valid scene_root
    *scene_root = commands.spawn_bundle(()).id();
}
