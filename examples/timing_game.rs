use bevy::{
    audio::Volume,
    input::keyboard::KeyboardInput,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

const WINDOW_SIZE: Vec2 = Vec2::new(800.0, 600.0);

const SLIDER_SIZE: Vec2 = Vec2::new(500.0, 50.0);

const REFLECTOR_SIZE: Vec2 = Vec2::new(1.0, 50.0);

const CUE_SIZE: Vec2 = Vec2::new(5.0, 50.0);
const CUE_SPEED: f32 = 500.0;
const INITIAL_CUE_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const PERFECT_TIMING_RANGE: f32 = 10.0;
const GOOD_TIMING_RANGE: f32 = 50.0;
const OK_TIMING_RANGE: f32 = 150.0;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const PRESSANYKEY_FONT_SIZE: f32 = 40.0;
const PRESSANYKEY_TEXT_PADDING: Val = Val::Px(20.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const SLIDER_DEFAULT_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const SLIDER_OK_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SLIDER_GOOD_COLOR: Color = Color::rgb(0.6, 0.6, 0.6);
const SLIDER_PERFECT_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
const REFRECTOR_COLOR: Color = Color::rgb(0.4, 0.4, 0.4);
const CUE_COLOR: Color = Color::rgb(0.4, 0.4, 0.4);
const PRESSANYKEY_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Default, States)]
enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(Resource, Component)]
struct Scoreboard {
    score: isize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WINDOW_SIZE.into(),
                ..default()
            }),
            ..default()
        }))
        .add_state::<AppState>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .insert_resource(Scoreboard { score: 0 })
        .add_systems(Startup, setup)
        .add_systems(Update, press_any_key.run_if(in_state(AppState::MainMenu)))
        .add_systems(Update, decide_timing.run_if(in_state(AppState::InGame)))
        .add_systems(
            Update,
            check_for_collisions.run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, apply_velocity.run_if(in_state(AppState::InGame)))
        .add_systems(Update, update_scoreboard.run_if(in_state(AppState::InGame)))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Cue;

#[derive(Component)]
struct Reflector;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct PressAnyKey;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Press any key
    commands.spawn((
        TextBundle::from_section(
            "Press Any Key ...",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: PRESSANYKEY_FONT_SIZE,
                color: PRESSANYKEY_COLOR,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: PRESSANYKEY_TEXT_PADDING,
            right: PRESSANYKEY_TEXT_PADDING,
            ..default()
        }),
        PressAnyKey,
    ));

    // Slider
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: SLIDER_DEFAULT_COLOR,
            custom_size: Some(SLIDER_SIZE),
            ..default()
        },
        ..default()
    });

    // Slider ok timing range
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: SLIDER_OK_COLOR,
            custom_size: Some(Vec2::new(OK_TIMING_RANGE * 2.0, SLIDER_SIZE.y)),
            ..default()
        },
        ..default()
    });

    // Slider good timing range
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: SLIDER_GOOD_COLOR,
            custom_size: Some(Vec2::new(GOOD_TIMING_RANGE * 2.0, SLIDER_SIZE.y)),
            ..default()
        },
        ..default()
    });

    // Slider parfect timing range
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: SLIDER_PERFECT_COLOR,
            custom_size: Some(Vec2::new(PERFECT_TIMING_RANGE * 2.0, SLIDER_SIZE.y)),
            ..default()
        },
        ..default()
    });

    let refrector_sprite = |slider_pos_x: f32| SpriteBundle {
        sprite: Sprite {
            color: REFRECTOR_COLOR,
            custom_size: Some(REFLECTOR_SIZE),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(slider_pos_x / 2.0, 0.0, 0.0),
            ..default()
        },
        ..default()
    };

    // left reflector
    commands.spawn((refrector_sprite(-SLIDER_SIZE.x), Reflector, Collider));

    // right reflector
    commands.spawn((refrector_sprite(SLIDER_SIZE.x), Reflector, Collider));

    // Cue
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: CUE_COLOR,
                custom_size: Some(CUE_SIZE),
                ..default()
            },
            ..default()
        },
        Cue,
        Velocity(INITIAL_CUE_DIRECTION.normalize() * CUE_SPEED),
    ));

    // Scoreboard
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            TextSection::new(
                "0",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: Color::GRAY,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
        Scoreboard { score: 0 },
    ));
}

fn press_any_key(
    mut keyboard_event: EventReader<KeyboardInput>,
    pressanykey_query: Query<Entity, With<PressAnyKey>>,
    mut commands: Commands,
    mut now_state: ResMut<State<AppState>>,
    mut inkey: ResMut<Input<KeyCode>>,
) {
    for _event in keyboard_event.iter() {
        let pressanykey_entity = pressanykey_query.single();
        commands.entity(pressanykey_entity).despawn();

        *now_state = State::new(AppState::InGame);
        inkey.reset_all();
    }
}

fn decide_timing(
    keyboard_input: Res<Input<KeyCode>>,
    mut scoreboard: ResMut<Scoreboard>,
    cue_query: Query<&Transform, With<Cue>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let cue_transform = cue_query.single();

    if keyboard_input.just_pressed(KeyCode::Space) {
        // Sends a timing event so that other systems can react to the timing
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/timing.ogg"),
            settings: PlaybackSettings::ONCE.with_volume(Volume::new_relative(0.5)),
        });

        let cue_translation_x = cue_transform.translation.x;

        if cue_translation_x < PERFECT_TIMING_RANGE && cue_translation_x > -PERFECT_TIMING_RANGE {
            scoreboard.score += 100;
        } else if cue_translation_x < GOOD_TIMING_RANGE && cue_translation_x > -GOOD_TIMING_RANGE {
            scoreboard.score += 50;
        } else if cue_translation_x < OK_TIMING_RANGE && cue_translation_x > -OK_TIMING_RANGE {
            scoreboard.score += 10;
        } else {
            scoreboard.score += -100;
        }
    }
}

fn check_for_collisions(
    mut cue_query: Query<(&mut Velocity, &Transform), With<Cue>>,
    collider_query: Query<&Transform, With<Collider>>,
) {
    let (mut cue_velocity, cue_transform) = cue_query.single_mut();

    // check collision with reflectors
    for transform in &collider_query {
        let collision = collide(
            cue_transform.translation,
            CUE_SIZE,
            transform.translation,
            transform.scale.truncate(),
        );

        if let Some(collision) = collision {
            let reflect_x = match collision {
                Collision::Left => cue_velocity.x > 0.0,
                Collision::Right => cue_velocity.x < 0.0,
                _ => false,
            };

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                cue_velocity.x = -cue_velocity.x;
            }
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time_step: Res<FixedTime>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time_step.period.as_secs_f32();
    }
}

fn update_scoreboard(
    scoreboard: Res<Scoreboard>,
    mut scoreboard_query: Query<&mut Text, With<Scoreboard>>,
) {
    let mut text = scoreboard_query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}
