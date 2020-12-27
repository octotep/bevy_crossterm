use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use std::default::Default;

mod animation;
mod colors;
mod finale;
mod sprites;
mod title;

#[derive(Clone)]
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

static STAGE: &str = "DEMO";

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = WindowSettings::default();
    settings.set_title("bevy_crossterm demo");

    App::build()
        .add_resource(settings)
        .add_resource(bevy::app::ScheduleRunnerSettings::run_loop(
            std::time::Duration::from_millis(16),
        ))
        .add_plugins(DefaultPlugins)
        .add_plugin(CrosstermPlugin)
        .add_startup_system(demo_setup.system())
        .add_startup_system(loading_system.system())
        .add_stage_after(stage::UPDATE, STAGE, StateStage::<GameState>::default())
        .add_resource(State::new(GameState::Loading))
        .on_state_update(STAGE, GameState::Loading, check_for_loaded.system())
        .on_state_enter(STAGE, GameState::Title, title::setup.system())
        .on_state_update(STAGE, GameState::Title, just_wait_and_advance.system())
        .on_state_exit(STAGE, GameState::Title, simple_teardown.system())
        .on_state_enter(STAGE, GameState::Sprites, sprites::setup.system())
        .on_state_update(STAGE, GameState::Sprites, just_wait_and_advance.system())
        .on_state_exit(STAGE, GameState::Sprites, simple_teardown.system())
        .on_state_enter(STAGE, GameState::Colors, colors::setup.system())
        .on_state_update(STAGE, GameState::Colors, just_wait_and_advance.system())
        .on_state_exit(STAGE, GameState::Colors, simple_teardown.system())
        .on_state_enter(STAGE, GameState::Animation, animation::setup.system())
        .on_state_update(STAGE, GameState::Animation, animation::update.system())
        .on_state_exit(STAGE, GameState::Animation, simple_teardown.system())
        .on_state_enter(STAGE, GameState::Finale, finale::setup.system())
        .on_state_update(STAGE, GameState::Finale, just_wait_and_advance.system())
        .on_state_exit(STAGE, GameState::Finale, simple_teardown.system())
        .run();
}

fn loading_system(
    commands: &mut Commands,
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
            let next_state = state.next_state().unwrap();
            state.set_next(next_state).unwrap();
        }
        bevy::asset::LoadState::Failed => {}
    }
}

// Setup anything needed globally for the demo
pub fn demo_setup(commands: &mut Commands) {
    let scene_root = commands.spawn(()).current_entity().unwrap();

    commands.insert_resource(scene_root);
}

// Helper function to see if there was a key press this frame
pub fn detect_keypress(keys: Res<Events<KeyEvent>>) -> bool {
    keys.get_reader().latest(&keys).is_some()
}

// Simple update function that most screens will use
pub fn just_wait_and_advance(
    mut state: ResMut<State<GameState>>,
    mut app_exit: ResMut<Events<bevy::app::AppExit>>,
    keys: Res<Events<KeyEvent>>,
) {
    if detect_keypress(keys) {
        if let Some(next_stage) = state.next_state() {
            state.set_next(next_stage).unwrap();
        } else {
            app_exit.send(bevy::app::AppExit);
        }
    }
}

// Looks for an entity resource and then despawns that entity and all it's children
pub fn simple_teardown(commands: &mut Commands, mut scene_root: ResMut<Entity>) {
    commands.despawn_recursive(*scene_root);

    // Create a new, valid scene_root
    *scene_root = commands.spawn(()).current_entity().unwrap();
}
