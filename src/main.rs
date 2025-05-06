use std::collections::HashSet;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    window::PrimaryWindow,
};
use components::{
    Enemy, Explosion, ExplosionTimer, FromEnemy, FromPlayer, Laser, MainMenu, Movable, Player,
    ScoreBoardUI, SpriteSize, Velocity,
};
use enemy::EnemyPlugin;
use player::PlayerPlugin;

mod components;
mod enemy;
mod player;

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144., 75.);
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.);

const MAX_ENEMIES: u32 = 10;

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize = 16;

const SPRITE_SCALE: f32 = 0.5;
const BASE_SPEED: f32 = 600.0;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GameState {
    #[default]
    Startup,
    MainMenu,
    Playing,
    GameOver,
}

#[derive(Resource)]
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}

#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion_layout: Handle<TextureAtlasLayout>,
    explosion_texture: Handle<Image>,
}

#[derive(Resource, Deref, DerefMut)]
struct Score(u32);

#[derive(Resource)]
struct EnemyCount(u32);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .insert_resource(Score(0))
        .insert_resource(EnemyCount(0))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Invaders!".into(),
                resolution: (800., 800.).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, game_over.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, start_game.run_if(in_state(GameState::MainMenu)))
        .add_systems(Update, movement)
        .add_systems(
            Update,
            player_laser_hit_enemy.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            enemy_laser_hit_player.run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, update_scoreboard)
        .add_systems(Update, explosion_animation)
        .init_state::<GameState>()
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<&Window, With<PrimaryWindow>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Text::new("Start Game [enter]\n\n\n a & d to move\n up-arrow to shoot"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(350.0),
            left: Val::Px(300.0),
            ..default()
        },
        MainMenu,
    ));

    commands.spawn((
        Text::new("Score: "),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        ScoreBoardUI,
        children![(TextSpan::default(),)],
    ));

    // capture window size
    let Ok(primary) = query.single() else {
        return;
    };
    let (win_w, win_h) = (primary.width(), primary.height());

    // add WinSize resource
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    // create explosion texture atlas
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(64, 64), 4, 4, None, None);
    let explosion_layout = texture_atlases.add(texture_atlas);

    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion_layout,
        explosion_texture: texture_handle,
    };

    commands.insert_resource(game_textures);
    next_state.set(GameState::MainMenu);
}

fn start_game(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    main_menu_query: Query<Entity, With<MainMenu>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if input.pressed(KeyCode::Enter) {
        for entity in &main_menu_query {
            commands.entity(entity).despawn();
        }
        **score = 0;
        next_state.set(GameState::Playing);
    }
}

fn game_over(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    explosion_query: Query<(), With<Explosion>>,
) {
    if explosion_query.iter().len() == 0 {
        commands.spawn((
            Text::new("You Died!\nGame Over\n\nrestart [enter]"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(350.0),
                left: Val::Px(350.0),
                ..default()
            },
            MainMenu,
        ));
        next_state.set(GameState::MainMenu);
    }
}

fn update_scoreboard(
    score: Res<Score>,
    score_root: Single<Entity, (With<ScoreBoardUI>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*score_root, 1) = score.to_string();
}

fn movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
    time: Res<Time>,
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        let delta = time.delta_secs();
        translation.x += velocity.x * delta * BASE_SPEED;
        translation.y += velocity.y * delta * BASE_SPEED;

        if movable.auto_despawn {
            let margin = 200.0;
            if translation.y > win_size.h / 2. + margin
                || translation.y < -win_size.h / 2. - margin
                || translation.x > win_size.w / 2. + margin
                || translation.x < -win_size.w / 2. - margin
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_laser_hit_enemy(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut enemy_count: ResMut<EnemyCount>,
    game_textures: Res<GameTextures>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    for (laser_entity, laser_tf, laser_size) in &laser_query {
        if despawned_entities.contains(&laser_entity) {
            continue;
        }

        let laser_scale = Vec2::from(laser_tf.scale.xy());

        for (enemy_entity, enemy_tf, enemy_size) in &enemy_query {
            if despawned_entities.contains(&enemy_entity)
                || despawned_entities.contains(&laser_entity)
            {
                continue;
            }

            let enemy_scale = Vec2::from(enemy_tf.scale.xy());

            let collision = Aabb2d::new(
                laser_tf.translation.truncate(),
                (laser_size.0 * laser_scale) / 2.0,
            )
            .intersects(&Aabb2d::new(
                enemy_tf.translation.truncate(),
                (enemy_size.0 * enemy_scale) / 2.0,
            ));

            if collision {
                despawned_entities.insert(enemy_entity);
                despawned_entities.insert(laser_entity);
                commands.entity(enemy_entity).despawn();
                commands.entity(laser_entity).despawn();
                // commands.spawn(ExplosionToSpawn(enemy_tf.translation));
                commands.spawn((
                    Sprite {
                        image: game_textures.explosion_texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: game_textures.explosion_layout.clone(),
                            index: 0,
                        }),
                        ..Default::default()
                    },
                    Transform::from_translation(enemy_tf.translation),
                    Explosion,
                    ExplosionTimer::default(),
                ));
                **score += 1;
                enemy_count.0 -= 1;
            }
        }
    }
}

fn enemy_laser_hit_player(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    for (laser_entity, laser_tf, laser_size) in &laser_query {
        if despawned_entities.contains(&laser_entity) {
            continue;
        }

        let laser_scale = Vec2::from(laser_tf.scale.xy());

        for (player_entity, player_tf, player_size) in &player_query {
            if despawned_entities.contains(&player_entity) {
                continue;
            }

            let player_scale = Vec2::from(player_tf.scale.xy());

            let collision = Aabb2d::new(
                laser_tf.translation.truncate(),
                (laser_size.0 * laser_scale) / 2.0,
            )
            .intersects(&Aabb2d::new(
                player_tf.translation.truncate(),
                (player_size.0 * player_scale) / 2.0,
            ));

            if collision {
                despawned_entities.insert(laser_entity);
                despawned_entities.insert(player_entity);
                commands.entity(laser_entity).despawn();
                commands.entity(player_entity).despawn();
                commands.spawn((
                    Sprite {
                        image: game_textures.explosion_texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: game_textures.explosion_layout.clone(),
                            index: 0,
                        }),
                        ..Default::default()
                    },
                    Transform::from_translation(player_tf.translation),
                    Explosion,
                    ExplosionTimer::default(),
                ));
                next_state.set(GameState::GameOver);
                break;
            }
        }
    }
}

fn explosion_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut Sprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            if let Some(texture) = sprite.texture_atlas.as_mut() {
                texture.index += 1;
                if texture.index >= EXPLOSION_LEN {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
