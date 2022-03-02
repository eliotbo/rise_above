use bevy::render::view::{ComputedVisibility, Visibility};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::{keyboard::KeyCode, Input},
    prelude::*,
    sprite::Material2dPlugin,
    sprite::MaterialMesh2dBundle,
    sprite::Mesh2dHandle,
};

// use bytemuck::{Pod, Zeroable};

use balls2d::*;
use bevy_inspector_egui::InspectorPlugin;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "I am a window!".to_string(),
            width: 600.,
            height: 900.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Material2dPlugin::<SelectionMaterial>::default())
        .add_plugin(MarkerMesh2dPlugin)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        .add_event::<SpawnAllEvent>()
        // .add_event::<DragEvent>()
        .add_plugin(SelectionMesh2dPlugin)
        .add_plugin(InspectorPlugin::<InstanceDataNotEncoded>::new())
        // .insert_resource(AssetServerSettings {
        //     watch_for_changes: true,
        //     ..Default::default()
        // })
        // .add_asset::<SelectionMaterial>()
        .insert_resource(Cursor::default())
        .add_startup_system(setup)
        .add_system(characters_setup)
        .add_system(move_characters)
        .add_system(record_mouse_events_system)
        .add_system(release)
        .add_system(drag)
        .add_system(summon)
        .add_system(update_time)
        .add_system(update_character.after("inspector"))
        .add_system(character_inspector_update.label("inspector"))
        .add_system(save_character)
        // .add_system(load_character)
        .add_system(delete_point)
        .run();
}

fn setup(
    mut commands: Commands,
    mut spawn_event: EventWriter<SpawnAllEvent>,
    asset_server: Res<AssetServer>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<SelectionMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    spawn_event.send(SpawnAllEvent);

    let text_style = TextStyle {
        font: asset_server.load("fonts/Roboto-Regular.ttf"),
        font_size: 25.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Bottom,
        horizontal: HorizontalAlign::Left,
    };

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("Selected: 0", text_style.clone(), text_alignment),
            transform: Transform::from_translation(Vec3::new(0.0, 250.0, 10.0)),
            ..Default::default()
        })
        .insert(SelectedText);

    let dy = -100.0;

    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            "Spawn Circle: Left Alt + Left Mouse Click (max of 44) 

            ",
            text_style.clone(),
            TextAlignment {
                vertical: VerticalAlign::Top,
                horizontal: HorizontalAlign::Left,
            },
        ),
        transform: Transform::from_translation(Vec3::new(-250.0, -250.0 + dy, 10.0)),
        ..Default::default()
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            "Spawn Joint: Left Alt + Right Mouse Click (max of 5)",
            text_style.clone(),
            TextAlignment {
                vertical: VerticalAlign::Top,
                horizontal: HorizontalAlign::Left,
            },
        ),
        transform: Transform::from_translation(Vec3::new(-250.0, -270.0 + dy, 10.0)),
        ..Default::default()
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            "Move Circle: Left Mouse Drag",
            text_style.clone(),
            TextAlignment {
                vertical: VerticalAlign::Top,
                horizontal: HorizontalAlign::Left,
            },
        ),
        transform: Transform::from_translation(Vec3::new(-250.0, -290.0 + dy, 10.0)),
        ..Default::default()
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            "Change Circle Size (max and min): Right Mouse Drag  ",
            text_style.clone(),
            TextAlignment {
                vertical: VerticalAlign::Top,
                horizontal: HorizontalAlign::Left,
            },
        ),
        transform: Transform::from_translation(Vec3::new(-250.0, -310.0 + dy, 10.0)),
        ..Default::default()
    });
}

pub fn f(mut x: f32) -> f32 {
    let freq = 15.0;
    x = x - 0.5;
    let y = (x * freq).sin() / 4.0 * (1.2 - x.abs()) + 0.3;
    return y;
}

pub fn update_time(time: Res<Time>, mut query: Query<(&mut CharacterUniform,)>) {
    for (mut character_uniform,) in query.iter_mut() {
        character_uniform.time = time.seconds_since_startup() as f32;
    }
}

pub struct SpawnAllEvent;

pub fn characters_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut spawn_event: EventReader<SpawnAllEvent>,

    query: Query<Entity, With<CharacterUniform>>,
    // mut materials: ResMut<Assets<SelectionMaterial>>,
) {
    for _event in spawn_event.iter() {
        //
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        spawn_character(&mut commands, &mut meshes);

        let mut material = SelectionMaterial::default();
        material.pos = Vec2::ZERO;
        material.radius = 10.0;

        // let canvas_material_handle = materials.add(material);

        // let plot_entity = commands
        //     .spawn()
        //     .insert_bundle(MaterialMesh2dBundle {
        //         mesh: Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::splat(100.0))))),
        //         material: canvas_material_handle.clone(),
        //         transform: Transform::from_translation(Vec2::ZERO.extend(2.0)),
        //         ..Default::default()
        //     })
        //     // .insert(SelectionHalo)
        //     .id();
    }
}

#[derive(Component)]
pub struct SelectionHalo;

#[derive(Component, Debug)]
pub struct DoDrag {
    pub index: usize,
    pub instance: usize,
    pub initial_pos: Vec2,
}

#[derive(Component, Debug)]
pub struct Summon {
    pub index: usize,
    pub instance: usize,
}

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub fn open_file_dialog(save_name: &str, folder: &str, extension: &str) -> Option<PathBuf> {
    let mut k: usize = 0;

    let mut default_path = std::env::current_dir().unwrap();
    default_path.push("saved");
    default_path.push(folder.to_string());
    let mut default_name: String;

    loop {
        default_name = save_name.to_string();
        default_name.push_str(&(k.to_string()));
        default_name.push_str(extension);

        default_path.push(&default_name);

        if !default_path.exists() {
            break;
        }
        default_path.pop();

        k += 1;
    }

    let res = rfd::FileDialog::new()
        .set_file_name(&default_name)
        .set_directory(&default_path)
        .save_file();
    println!("The user choose: {:#?}", &res);

    return res;
}

// fn save_character(
//     keyboard: Res<Input<KeyCode>>,
//     mut query: Query<(&mut MarkerInstanceMatData, &CharacterUniform)>,
// ) {
//     if keyboard.pressed(KeyCode::LControl) && keyboard.just_pressed(KeyCode::S) {
//         // convert MarkerInstanceMatData to CharacterSaveFormat
//         for (character, _unif) in query.iter_mut() {
//             //
//             let character2 = character.as_ref().clone();
//             let data: CharacterSaveFormat = character2.into();

//             // let lut_dialog_result =
//             //     open_file_dialog("my_character", "character_save_format", ".cha");

//             // if let Some(lut_path) = lut_dialog_result {
//             let lut_serialized = serde_json::to_string_pretty(&data).unwrap();

//             // let mut lut_output = File::create(&lut_path).unwrap();
//             let mut cwd = std::env::current_dir().unwrap();
//             cwd = cwd.join("damn.cha");
//             println!("{:?}", cwd);
//             let mut lut_output = File::create(cwd).unwrap();
//             let _lut_write_result = lut_output.write(lut_serialized.as_bytes());
//             // }
//         }
//     }
// }

fn save_character(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut MarkerInstanceMatData, &CharacterUniform)>,
) {
    if keyboard.pressed(KeyCode::LControl) && keyboard.just_pressed(KeyCode::S) {
        // convert MarkerInstanceMatData to CharacterSaveFormat
        for (character, _unif) in query.iter_mut() {
            //
            let character2 = character.as_ref().clone();
            let data: CharacterSaveFormat = character2.into();

            let lut_dialog_result =
                open_file_dialog("my_character", "character_save_format", ".cha");
            if let Some(lut_path) = lut_dialog_result {
                let lut_serialized = serde_json::to_string_pretty(&data).unwrap();

                let mut lut_output = File::create(&lut_path).unwrap();
                let _lut_write_result = lut_output.write(lut_serialized.as_bytes());
            }
        }
    }
}

pub fn load_character(keyboard: Res<Input<KeyCode>>, mut query: Query<&mut MarkerInstanceMatData>) {
    if keyboard.pressed(KeyCode::LControl) && keyboard.just_pressed(KeyCode::L) {
        let default_path = std::env::current_dir().unwrap();
        // default_path.push("saved");
        // default_path.push("groups");

        let res = rfd::FileDialog::new()
            .add_filter("text", &["cha"])
            .set_directory(&default_path)
            .pick_files();

        // cancel loading if user cancelled the file dialog
        let path: std::path::PathBuf;
        if let Some(chosen_path) = res.clone() {
            let path_some = chosen_path.get(0);
            if let Some(path_local) = path_some {
                path = path_local.clone();
            } else {
                return ();
            }
        } else {
            return ();
        }

        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        println!("loaded: {}", contents);

        let loaded_character: CharacterSaveFormat = serde_json::from_str(&contents).unwrap();

        for mut character in query.iter_mut() {
            let character2 = character.as_mut();
            *character2 = loaded_character.clone().into();
        }
    }
}

fn character_inspector_update(
    mut query: Query<&mut InstanceDataNotEncoded>,
    inspector_data: Res<InstanceDataNotEncoded>,
) {
    if inspector_data.is_changed() {
        for mut instance_data in query.iter_mut() {
            *instance_data = inspector_data.clone();
        }
    }
}

fn update_character(
    // mut commands: Commands,
    mut query: Query<
        (
            // Entity,
            &mut MarkerInstanceMatData,
            &mut InstanceDataNotEncoded,
            &SelectedBall,
        ),
        Changed<InstanceDataNotEncoded>,
    >,
) {
    //

    for (mut character_instance_mat_data, instance_data_not_encoded, selected) in query.iter_mut() {
        if let Some(selected_index) = selected.ball_index {
            character_instance_mat_data.0[selected.instance]
                .set_data(selected_index, instance_data_not_encoded.clone());
            println!("changed");
        }

        // println!("selected :{}", instance_data_not_encoded.pos);
        // println!("selected :{:?}", instance_data_not_encoded);
    }
}

fn move_characters(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut MarkerInstanceMatData,
        &CharacterUniform,
        &mut SelectedBall,
    )>,
    halo_query: Query<(Entity, &SelectionHalo)>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    cursor: Res<Cursor>,
    mut materials: ResMut<Assets<SelectionMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut inspector_data: ResMut<InstanceDataNotEncoded>,
    mut text_query: Query<&mut Text, With<SelectedText>>,
    // mut drag_event_writer: EventWriter<DragEvent>,
) {
    // if mouse_button_input.just_pressed(MouseButton::Left) {
    //     println!("c: {}", cursor.position);
    //     println!(" ");
    // }

    for (entity, _) in halo_query.iter() {
        commands.entity(entity).despawn();
    }

    for (character_entity, mut character, unif, mut selected) in query.iter_mut() {
        for (instance_number, data) in character.0.iter_mut().enumerate() {
            // if mouse_button_input.just_pressed(MouseButton::Left) {
            // for k in 0..data.color.len() / 2 - 1 {
            for k in 0..ATTR_SIZE * 2 {
                let v = data.get_pos(k) * unif.quad_size
                    + unif.canvas_position
                    + data.get_group_position().truncate();

                // println!("vL {}", cursor.position);
                if (v - cursor.position).length() < 10.0 {
                    // println!("v: {}", v);
                    // println!("c: {}", cursor.position);

                    let mut material = SelectionMaterial::default();
                    material.pos = v;
                    material.radius = 10.0;

                    let canvas_material_handle = materials.add(material);

                    // // uncomment to draw a circle around the hovered ball (does not work with wasm)
                    // // quad
                    // let _plot_entity = commands
                    //     .spawn()
                    //     .insert_bundle(MaterialMesh2dBundle {
                    //         mesh: Mesh2dHandle(
                    //             meshes.add(Mesh::from(shape::Quad::new(Vec2::splat(100.0)))),
                    //         ),
                    //         material: canvas_material_handle.clone(),
                    //         transform: Transform::from_translation(v.extend(2.0)),
                    //         ..Default::default()
                    //     })
                    //     .insert(SelectionHalo)
                    //     .id();

                    if mouse_button_input.just_pressed(MouseButton::Left)
                        && !keyboard.pressed(KeyCode::LAlt)
                    {
                        // let p = (cursor.position
                        //     - unif.canvas_position
                        //     - data.group_position.truncate())
                        //     / unif.quad_size;
                        selected.ball_index = Some(k);

                        println!("selected: {}", k);

                        for mut text in text_query.iter_mut() {
                            // text.text = format!("{}", k);
                            text.sections[0].value = format!("Selection: {}", k);
                        }

                        let datas = data.get_all(k);
                        *inspector_data = datas;

                        let pos = data.get_pos(k);

                        commands.entity(character_entity).insert(DoDrag {
                            index: k,
                            instance: instance_number,
                            initial_pos: pos,
                            // initial_pos: data.data[k],
                        });
                    }

                    break;
                }
            }

            // // brownian motion
            // if mouse_button_input.pressed(MouseButton::Left) {
            //     // data.group_position += Vec3::new(0.001, 0.0, 0.00) * 0.0;
            //     let rng: usize = rand::thread_rng().gen_range(4, 22);
            //     let rng2: f32 = rand::thread_rng().gen();
            //     data.data[rng] += (rng2 - 0.5) * 0.01;
            // }
        }
    }
}

fn summon(
    // mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut MarkerInstanceMatData,
        &mut Summon,
        &CharacterUniform,
        &mut SelectedBall,
    )>,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    keyboard: Res<Input<KeyCode>>,
    mut text_query: Query<&mut Text, With<SelectedText>>,
    mut inspector_data: ResMut<InstanceDataNotEncoded>,
) {
    //
    if mouse_button_input.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::LAlt) {
        for (_character_entity, mut character, mut summon, unif, mut selected) in query.iter_mut() {
            if let Some(instance_number) = character.within_rect(cursor.position, unif.quad_size) {
                let character_pos = character.0[instance_number].get_group_position();

                let mut not_encoded_new_circle = InstanceDataNotEncoded::default();

                let pos = (cursor.position - character_pos.truncate()) / unif.quad_size;

                // character.0[instance_number].set_pos(p, summon.index);
                not_encoded_new_circle.pos = pos;

                let mut inc = 0;
                // find next unused index

                loop {
                    summon.index += 1;
                    summon.index %= balls2d::ATTR_SIZE * 2;

                    let character_unencoded = character.0[instance_number].get_all(summon.index);
                    let c_pos = character_unencoded.pos * unif.quad_size + character_pos.truncate();

                    println!("c_pos: {}", c_pos);
                    println!(
                        "character_pos.x + unif.quad_size / 2.0 * 0.99: {}",
                        character_pos.x + unif.quad_size / 2.0 * 0.99
                    );

                    if !(c_pos.x < character_pos.x + unif.quad_size / 2.0 * 0.99
                        && c_pos.x > character_pos.x - unif.quad_size / 2.0 * 0.99
                        && c_pos.y < character_pos.y + unif.quad_size / 2.0 * 0.99
                        && c_pos.y > character_pos.y - unif.quad_size / 2.0 * 0.99)
                    {
                        selected.ball_index = Some(summon.index);

                        character.0[instance_number].set_data(summon.index, not_encoded_new_circle);

                        *inspector_data = not_encoded_new_circle;

                        for mut text in text_query.iter_mut() {
                            // text.text = format!("{}", k);
                            text.sections[0].value = format!("Selection: {}", summon.index);
                        }

                        break;
                    // break out if we have checked all indices
                    } else {
                        inc += 1;
                    }

                    if inc > ATTR_SIZE * 2 {
                        println!("All circles are used already");
                        break;
                    }
                }

                println!("broke out with {}", summon.index);
            }
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Right) && keyboard.pressed(KeyCode::LAlt) {
        for (_character_entity, mut character, mut summon, unif, mut selected) in query.iter_mut() {
            if let Some(instance_number) = character.within_rect(cursor.position, unif.quad_size) {
                let character_pos = character.0[instance_number].get_group_position();

                // let mut not_encoded_new_circle = InstanceDataNotEncoded::default();

                let pos = (cursor.position - character_pos.truncate()) / unif.quad_size;

                let mut not_encoded_new_circle = InstanceDataNotEncoded::new_joint_at_pos(pos);

                // character.0[instance_number].set_pos(p, summon.index);
                not_encoded_new_circle.pos = pos;

                let mut inc = 0;
                // find next unused index

                loop {
                    summon.index += 1;
                    summon.index %= ATTR_SIZE * 2;

                    let character_unencoded = character.0[instance_number].get_all(summon.index);

                    let c_pos = character_unencoded.pos * unif.quad_size + character_pos.truncate();

                    println!("c_pos: {}", c_pos);
                    println!(
                        "character_pos.x + unif.quad_size / 2.0 * 0.99: {}",
                        character_pos.x + unif.quad_size / 2.0 * 0.99
                    );

                    if !(c_pos.x < character_pos.x + unif.quad_size / 2.0 * 0.99
                        && c_pos.x > character_pos.x - unif.quad_size / 2.0 * 0.99
                        && c_pos.y < character_pos.y + unif.quad_size / 2.0 * 0.99
                        && c_pos.y > character_pos.y - unif.quad_size / 2.0 * 0.99)
                    {
                        selected.ball_index = Some(summon.index);

                        character.0[instance_number].set_data(summon.index, not_encoded_new_circle);

                        *inspector_data = not_encoded_new_circle;

                        for mut text in text_query.iter_mut() {
                            // text.text = format!("{}", k);
                            text.sections[0].value = format!("Selection: {}", summon.index);
                        }

                        break;
                    // break out if we have checked all indices
                    } else {
                        inc += 1;
                    }

                    if inc > ATTR_SIZE * 2 {
                        println!("All circles are used already");
                        break;
                    }
                }

                println!("broke out with {}", summon.index);
            }
        }
    }
}

fn delete_point(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MarkerInstanceMatData, &mut SelectedBall)>,
    keyboard: Res<Input<KeyCode>>,
) {
    for (character_entity, mut character, mut selected) in query.iter_mut() {
        if keyboard.pressed(KeyCode::Delete) {
            let maybe_ball_index = selected.ball_index.clone();

            if let Some(index) = maybe_ball_index {
                character.0[0].delete(index);

                commands.entity(character_entity).remove::<DoDrag>();
                selected.ball_index = None;
                break;
            }
        }
    }
}

fn drag(
    // mut commands: Commands,
    mut query: Query<(&mut MarkerInstanceMatData, &CharacterUniform, &DoDrag)>,
    // mouse_button_input: Res<Input<MouseButton>>,
    // keyboard: Res<Input<KeyCode>>,
    mut inspector: ResMut<InstanceDataNotEncoded>,
    cursor: Res<Cursor>,
) {
    for (mut character, unif, do_drag) in query.iter_mut() {
        // let mut data = character.0[do_drag.instance];

        let drag_dist = (cursor.pos_relative_to_click) / unif.quad_size * 1.0;

        character.0[do_drag.instance].set_pos(do_drag.initial_pos + drag_dist, do_drag.index);
        // println!("pos: {:?}", do_drag.initial_pos + drag_dist);
        inspector.pos = do_drag.initial_pos + drag_dist;
    }
}

fn release(
    mut commands: Commands,
    query: Query<(Entity, &DoDrag)>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for (entity, _) in query.iter() {
            commands.entity(entity).remove::<DoDrag>();
        }
    }
}

// const ATTR_SIZE: usize = 32;

// stress test: 8000 * (100 x 100) => limit for 60 fps. This means 320,000 circles
fn spawn_character(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>) {
    let quad_size = 500.0;

    let num_circles = 10;
    let num_joints = 3;

    let not_encoded = InstanceDataNotEncoded::default();
    // let vs = [InstanceDataNotEncoded::default].repeat(44);
    let mut vs = [not_encoded].repeat(num_circles + num_joints);

    let num_balls = 10;
    (0..num_balls).for_each(|k| {
        let theta = (k as f32 / num_balls as f32) * std::f32::consts::PI * 2.0 + 0.001;
        let pos = 0.35 * Vec2::new(theta.cos(), theta.sin());
        vs[k].pos = pos;
    });

    (num_balls..(num_balls + num_joints)).for_each(|k| {
        let theta = (k as f32 / num_balls as f32) * std::f32::consts::PI * 2.0 + 0.001;
        let pos = 0.25 * Vec2::new(theta.cos(), theta.sin());

        vs[k] = InstanceDataNotEncoded::new_joint_at_pos(pos);
    });

    let instance_data_vec = MarkerInstanceMatData(vec![MarkerInstanceData::new(vs)]);

    let quad_position = Vec2::new(0.0, 0.0);

    commands
        .spawn_bundle((
            Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
                size: Vec2::splat(quad_size),
                flip: false,
            }))),
            GlobalTransform::default(),
            Transform::from_translation(Vec3::new(quad_position.x, quad_position.y, 1.12)),
            Visibility::default(),
            ComputedVisibility::default(),
            instance_data_vec,
            // NoFrustumCulling,
        ))
        .insert(InstanceDataNotEncoded::default())
        .insert(SelectedBall::default())
        .insert(Summon {
            index: num_circles + num_joints,
            instance: 0,
        })
        .insert(CharacterUniform {
            character_size: 0.1,
            core_size: 1.0,
            zoom: 1.0,
            time: 0.0,
            character_point_color: Vec4::new(0.0, 1.0, 0.0, 1.0),
            color: Color::hex("8c114a").unwrap().into(),
            quad_size,
            inner_canvas_size_in_pixels: Vec2::new(300.0, 300.0),
            // outer_border: plot.outer_border,
            canvas_position: quad_position,
            contour: 1.0,
        });
}
