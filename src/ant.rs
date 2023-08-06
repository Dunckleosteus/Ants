use std::time::Duration;
extern crate random_choice;
use self::random_choice::random_choice;
use crate::food::Food;
use crate::Fenetre;
use crate::{chemin::Chemin, food::FoodId};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_prototype_lyon::prelude::*;
use rand::{thread_rng, Rng};
// timers
#[derive(Component)]
struct LineTime {
    time: Timer,
}
// components
#[derive(Component)]
pub struct PlacesTravelled {
    // places is optional because the ant may not have travelled yet
    places: Vec<i32>,
}
#[derive(Component)]
pub struct FinishedPath {
    pub finished: bool,
}
#[derive(Component)]
pub struct Path {
    pub points: Vec<Vec2>,
}
#[derive(Component)]
pub struct Line;
pub struct AntPlugin;
#[derive(Component)]
// plugin
pub struct Ant {
    pub target: Vec2,
}
impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_startup_system(setup.in_base_set(StartupSet::Startup))
            .add_startup_system(spawn_ant.in_base_set(StartupSet::PostStartup)) // can only happen once the food has spawned
            .add_system(choose_random_point)
            .add_system(draw_lines)
            .add_system(ant_move)
            .add_system(render_ant_path)
            .add_system(display_path_distance)
            .add_system(reset_path);
    }
}
// spawns timers and texts used by the ant plugin
fn setup(mut commands: Commands) {
    // spawning timers
    commands.spawn(LineTime {
        time: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
    });
}

fn spawn_ant(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    food: Query<(&Transform, &FoodId), With<Food>>, // choose a random transform from food list
) {
    let num_ants = 10;
    for _fourmi in 0..num_ants {
        /* This function spawns an ant on a radom food item */
        // TODO: Add a loop to chose how many  ants should be
        // selecting a random food transform, to be set as the ants coordinates
        let rand_trans: Vec<(&Transform, &FoodId)> = food.iter().collect();
        let food_ammount = rand_trans.len();
        let mut rng = thread_rng();
        let random_index = rng.gen_range(0..food_ammount);
        let x = rand_trans[random_index].0.translation.x;
        let y = rand_trans[random_index].0.translation.y;
        let food_rand_index = rand_trans[random_index].1.id;
        // spawning ant
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(10.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::GREEN)),
                transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 5.)),
                ..default()
            })
            .insert(Ant {
                target: Vec2::new(x as f32, y as f32),
            })
            .insert(Path {
                points: vec![Vec2::new(x as f32, y as f32)],
            })
            .insert(FinishedPath { finished: false })
            .insert(PlacesTravelled {
                places: vec![food_rand_index],
            });
    }
}

fn draw_lines(
    window: Res<Fenetre>,
    mut commands: Commands,
    ants: Query<(&Transform, &PlacesTravelled), With<Ant>>,
    foods: Query<(&Transform, &Food, &FoodId)>,
    mut lines: Query<Entity, With<Line>>,
) {
    let max_distance = (window.width.powf(2.) + window.heigth.powf(2.)).sqrt(); // the larges posible distance is a diagonal accross the screen
                                                                                // delete lines
    for line in lines.iter_mut() {
        commands.entity(line).despawn();
    }
    //
    for (ant, places) in ants.iter() {
        let finish = places.places.clone();
        for (food, food2, id) in foods.iter().filter(|x| finish.contains(&x.2.id) == false) {
            match food2.travelled {
                true => { /*The line has already been travelled to*/ }
                false => {
                    // the line has not been travelled to so draw a line
                    let distance_between_points = ((ant.translation.x - food.translation.x)
                        .powf(2.)
                        + (ant.translation.y - food.translation.y).powf(2.))
                    .sqrt();
                    let color_grad = distance_between_points / max_distance;
                    let width_grad = (5.0 - (distance_between_points / max_distance) * 5.) / 2.;
                    // building line start
                    let mut path_builder = PathBuilder::new();
                    path_builder.line_to(Vec2::new(ant.translation.x, ant.translation.y));
                    path_builder.line_to(Vec2::new(food.translation.x, food.translation.y));
                    path_builder.close();
                    let path = path_builder.build();
                    // building line end
                    // spawning line
                    let color = Color::rgba(color_grad, 1., 1., 1. - color_grad);
                    commands
                        .spawn((
                            ShapeBundle { path, ..default() },
                            Stroke::new(color, width_grad),
                        ))
                        .insert(Line);
                }
            }
        }
    }
}

fn choose_random_point(
    mut foods: Query<(&mut Food, &Transform, &FoodId)>,
    mut ants: Query<(&mut Ant, &Transform, &mut FinishedPath, &PlacesTravelled)>,
    time: Res<Time>,
    mut timer: Query<&mut LineTime>,
) {
    // this timer is used to time set how long an ant will wait before moving to the next point
    timer.get_single_mut().unwrap().time.tick(time.delta());
    if timer.get_single_mut().unwrap().time.just_finished() {
        for (mut ant, ant_pos, mut finished, places) in ants.iter_mut() {
            // filter foods so as to only keep the points that have not been visited yet
            // TODO: Change so that the ant choses based on it's internal destination counter
            let finish = places.places.clone();
            let food_filter = |x: &(&Food, &Transform, &FoodId)| finish.contains(&x.2.id) == false;
            let food_list = foods.iter().filter(food_filter);
            // the number of points that have not been visited yet
            let food_ammount = food_list.count();
            // create distance list as a weighted list
            if food_ammount > 0 {
                // create a distance list food <-> and
                let mut distance_list = foods
                    .iter()
                    .filter(food_filter)
                    .map(|(_y, x, _z)| {
                        (ant_pos.translation.x - x.translation.x).powf(2.)
                            + (ant_pos.translation.y - x.translation.y).powf(2.)
                    })
                    .collect::<Vec<f32>>();
                // find the maximum distance in distance_list
                let max_distance = distance_list
                    .iter()
                    .max_by(|&a, &b| a.partial_cmp(b).unwrap())
                    .unwrap();
                // flip distance values
                distance_list = distance_list.iter().map(|x| (max_distance - x)).collect();
                // chose random value based on distance -> usize
                let indexes: Vec<i32> = (0..food_ammount as i32).collect();
                let random_value = *random_choice().random_choice_f32(
                    &indexes[0..food_ammount],
                    &distance_list[0..food_ammount],
                    1,
                )[0] as usize;

                // updating ant target
                for (num, food) in foods
                    .iter_mut()
                    .filter(|x| finish.contains(&x.2.id) == false)
                    .enumerate()
                {
                    if num == random_value {
                        //food.0.travelled = true;
                        ant.target = Vec2::new(food.1.translation.x, food.1.translation.y);
                    }
                }
            } else {
                println!("all point have been visited");
                for (mut food, _transform, id) in foods.iter_mut() {
                    food.travelled = false;
                    finished.finished = true;
                }
            }
        }
    }
}

fn render_ant_path(mut path: Query<&mut Path, With<Ant>>, mut commands: Commands) {
    for ant in path.iter_mut() {
        let mut path_builder = PathBuilder::new();
        for segment in ant.points.iter() {
            path_builder.line_to(Vec2::new(segment.x, segment.y));
        }
        path_builder.close();
        let path = path_builder.build();
        // spawning line
        let color = Color::GREEN;
        commands
            .spawn((
                ShapeBundle {
                    path,
                    transform: Transform::from_xyz(0., 0., 5.),
                    ..default()
                },
                Stroke::new(color, 2.),
            ))
            .insert(Line);
    }
}

fn display_path_distance(paths: Query<&Path, With<Ant>>) {
    // calculates sum distance of all segments and displays it on the screen
    for path in paths.iter() {
        let points = path.points.clone();
        let mut points2 = points.clone();
        points2.rotate_right(1);
        let _total_distance: f32 = points
            .iter()
            .zip(points2.iter())
            .map(|(a, b)| ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0)).sqrt())
            .sum();
        // displaying result to the screen
        // TODO: Display the calculated distance on the screeen
    }
}
fn reset_path(
    mut ant: Query<
        (
            &mut Path,
            &mut FinishedPath,
            &Transform,
            &mut PlacesTravelled,
        ),
        Without<Food>,
    >,
    foods: Query<(&FoodId, &Transform), With<Food>>,
    mut commands: Commands,
) {
    // this function resets the ant's path and sets finished to false so that he can restart
    for (mut antpath, mut finished, &transform, mut places) in ant.iter_mut() {
        // if the ant's finished parameter is true then gotta
        match finished.finished {
            true => {
                // adding a chemin object to be rendered
                let che = antpath
                    .points
                    .iter()
                    .map(|x| Vec2::new(x.x, x.y))
                    .collect::<Vec<Vec2>>();

                // calculate length
                let points = antpath.points.clone();
                let mut points2 = points.clone();
                points2.rotate_right(1);
                let total_distance: f32 = points
                    .iter()
                    .zip(points2.iter())
                    .map(|(a, b)| ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0)).sqrt())
                    .sum();

                // spawning new path to screen from the che variable
                let mut path_builder = PathBuilder::new();
                for segment in che.iter() {
                    path_builder.line_to(Vec2::new(segment.x, segment.y));
                }
                path_builder.close();
                let path = path_builder.build();

                // spawning line
                let color = Color::ORANGE;
                commands
                    .spawn((
                        ShapeBundle {
                            path,
                            transform: Transform::from_xyz(0., 0., 6.),
                            ..default()
                        },
                        Stroke::new(color, 4.),
                    ))
                    .insert(Chemin {
                        dist: total_distance,
                    });
                // setting the ant's finished parameter to false so that it restarts and resetting distance path
                // have to put the ant on a random point

                antpath.points = vec![Vec2::new(transform.translation.x, transform.translation.y)];
                finished.finished = false;

                // resetting places the ant has travelled to
                let food_id: Option<i32> = foods
                    .iter()
                    .filter(|x| {
                        x.1.translation.x == transform.translation.x
                            && x.1.translation.y == transform.translation.y
                    })
                    .map(|x| x.0.id)
                    .next()
                    .map(|val| Some(val))
                    .unwrap_or(None);
                match food_id {
                    Some(t) => places.places = vec![t],
                    _ => {}
                }
            }
            false => {}
        }
    }
}
fn ant_move(
    mut ants: Query<(&Ant, &mut Transform, &mut Path, &mut PlacesTravelled), Without<Food>>,
    foods: Query<(&FoodId, &Transform), With<Food>>,
) {
    // the ant's target was defined in the choose random points function, now it has to travel to that point
    for (ant, mut position, mut path, mut places) in ants.iter_mut() {
        if (ant.target.x != position.translation.x) && (ant.target.y != position.translation.y) {
            // set the ant's new coordinates
            position.translation.x = ant.target.x;
            position.translation.y = ant.target.y;
            path.points
                .push(Vec2::new(position.translation.x, position.translation.y));
            // updating travelled id

            let food_id: Option<i32> = foods
                .iter()
                .filter(|x| x.1.translation.x == ant.target.x && x.1.translation.y == ant.target.y)
                .map(|x| x.0.id)
                .next()
                .map(|val| Some(val))
                .unwrap_or(None);
            match food_id {
                Some(t) => places.places.push(t),
                _ => {}
            }
        }
    }
}
