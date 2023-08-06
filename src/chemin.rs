use bevy::prelude::*;
// structs
#[derive(Resource)]
pub struct DeltaLen {
    value: Option<Vec<i32>>,
}
#[derive(Component)]
pub struct Chemin {
    pub dist: f32,
}
pub struct CheminPlugin;
impl Plugin for CheminPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(only_keep_shortest);
    }
}
fn setup(mut commands: Commands) {
    commands.insert_resource(DeltaLen { value: None });
}
fn only_keep_shortest(
    mut dists: Query<(Entity, &mut Chemin)>,
    mut commands: Commands,
    mut lon: ResMut<DeltaLen>,
) {
    let distances: Vec<i32> = dists.iter().map(|x| x.1.dist as i32).collect();
    let min_distance = distances.iter().min();
    match min_distance {
        Some(val) => {
            for (entity, path) in dists.iter_mut() {
                if *val != path.dist as i32 {
                    commands.entity(entity).despawn();
                }
            }
            let val2 = val.clone();

            if let Some(lon_value) = &mut lon.value {
                lon_value.push(val2);
            } else {
                lon.value = Some(vec![val2]);
            }
        }
        None => {}
    }
}
