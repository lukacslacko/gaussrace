//! Gaussian splat file loading and management

use bevy::prelude::*;
use bevy_gaussian_splatting::{GaussianScene, GaussianSceneHandle};

/// Plugin for loading and managing Gaussian splat files
pub struct SplatLoaderPlugin;

impl Plugin for SplatLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SplatLoadState>()
            .add_systems(Startup, load_from_cli_args)
            .add_systems(Update, (
                load_splat_on_demand,
                handle_file_drop,
            ))
            .add_systems(Update, check_splat_loaded.run_if(in_state(SplatLoadState::Loading)));
    }

}

/// Represents the state of splat loading
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SplatLoadState {
    #[default]
    WaitingForPath,
    Loading,
    Loaded,
    Failed,
}

/// Component marking the currently loaded splat entity
#[derive(Component)]
pub struct LoadedSplat;

/// Resource to hold the path of the splat file to load
#[derive(Resource)]
pub struct SplatPath(pub String);

/// System to load a splat when a path is provided
fn load_splat_on_demand(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    splat_path: Option<Res<SplatPath>>,
    mut next_state: ResMut<NextState<SplatLoadState>>,
    current_state: Res<State<SplatLoadState>>,
    existing_splats: Query<Entity, With<LoadedSplat>>,
) {
    // Only process in WaitingForPath state when we have a path
    if *current_state.get() != SplatLoadState::WaitingForPath {
        return;
    }

    if let Some(path) = splat_path {
        // Remove any existing splat
        for entity in existing_splats.iter() {
            commands.entity(entity).despawn();
        }

        // Load the new splat
        let handle: Handle<GaussianScene> = asset_server.load(&path.0);
        
        commands.spawn((
            GaussianSceneHandle(handle),
            Transform::default(),
            LoadedSplat,
        ));

        next_state.set(SplatLoadState::Loading);
        info!("Loading Gaussian splat from: {}", path.0);
    }
}

/// Check if the splat has finished loading
fn check_splat_loaded(
    asset_server: Res<AssetServer>,
    splat_query: Query<&GaussianSceneHandle, With<LoadedSplat>>,
    mut next_state: ResMut<NextState<SplatLoadState>>,
) {
    for handle in splat_query.iter() {
        match asset_server.get_load_state(handle.0.id()) {
            Some(bevy::asset::LoadState::Loaded) => {
                info!("Gaussian splat loaded successfully!");
                next_state.set(SplatLoadState::Loaded);
            }
            Some(bevy::asset::LoadState::Failed(_)) => {
                error!("Failed to load Gaussian splat!");
                next_state.set(SplatLoadState::Failed);
            }
            _ => {}
        }
    }
}

/// Load splat from command line arguments
fn load_from_cli_args(
    mut commands: Commands,
    mut next_state: ResMut<NextState<SplatLoadState>>,
) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let path = args[1].clone();
        info!("Found CLI argument, loading splat: {}", path);
        commands.insert_resource(SplatPath(path));
        next_state.set(SplatLoadState::WaitingForPath);
    }
}

/// Handle file drag and drop events
fn handle_file_drop(
    mut events: EventReader<FileDragAndDrop>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<SplatLoadState>>,
) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let path = path_buf.to_string_lossy().to_string();
            info!("File dropped, loading splat: {}", path);
            commands.insert_resource(SplatPath(path));
            // Reset state to trigger loading
            next_state.set(SplatLoadState::WaitingForPath);
        }
    }
}
