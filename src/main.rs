use ant::AntPlugin;
use bevy::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use chemin::CheminPlugin;
use food::FoodPlugin;
mod ant;
mod chemin;
mod components;
mod food;
// Assets
#[derive(Resource)]
pub struct Fenetre {
    pub width: f32,
    pub heigth: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FoodPlugin)
        .add_plugin(AntPlugin)
        .add_plugin(FramepacePlugin)
        .add_plugin(CheminPlugin)
        .add_startup_system(setup.in_base_set(StartupSet::PreStartup))
        .run();
}
fn setup(
    mut settings: ResMut<FramepaceSettings>,
    mut windows: Query<&mut Window>,
    mut commands: Commands,
) {
    settings.limiter = Limiter::from_framerate(30.);
    // spawn camera
    commands.spawn(Camera2dBundle::default());
    // adding window dimensions as ressources
    match windows.get_single_mut() {
        Ok(mut window) => {
            window.resizable = false;
            let width = window.resolution.physical_width() as f32;
            let height = window.resolution.physical_height() as f32;
            println!("width => {:?}, height => {:?}", width, height);
            commands.insert_resource(Fenetre {
                width: width,
                heigth: height,
            });
            window.title = "Blob".to_string();
        }
        Err(_e) => println!("Could not add screen dimensions as a ressource"),
    }
}
