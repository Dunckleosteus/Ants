use crate::Fenetre;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::{thread_rng, Rng};
// structs
#[derive(Component, Debug)]
pub struct Food {
    pub travelled: bool,
}
#[derive(Component)]
pub struct FoodId {
    pub id: i32,
}
// systems
fn add_food(
    window: Res<Fenetre>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    // this function will food randomly inside the map
    let food_ammount = 100;
    let mut rng = thread_rng();
    // create as many food items as food ammount variable
    for i in 0..food_ammount {
        let x = rng.gen_range(((-window.width / 2.) as i32)..((window.width / 2.) as i32));
        let y = rng.gen_range((-window.heigth / 2.0) as i32..(window.heigth / 2.) as i32);
        // spawn circle
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(10.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 2.)),
                ..default()
            })
            .insert(Food { travelled: false }) // TODO: Remove this as soon as possible
            .insert(FoodId { id: i as i32 }); // i is added to id so that the ant may remember where it has been
    }
}
fn paint_point(
    food_query: Query<(&Food, &mut Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // this function repaints the point if it's travelled attribute is true
    for (food, color_material) in food_query.iter() {
        if food.travelled == true {
            let mut material = materials.get_mut(color_material).unwrap();
            material.color = Color::RED;
        } else {
            let mut material = materials.get_mut(color_material).unwrap();
            material.color = Color::WHITE;
        }
    }
}
// Plugin
pub struct FoodPlugin;
impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_food).add_system(paint_point);
    }
}
