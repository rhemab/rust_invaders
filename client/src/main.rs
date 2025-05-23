#![windows_subsystem = "windows"]

use std::{collections::HashSet, fs, io, path::PathBuf};

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    window::PrimaryWindow,
};
use components::{
    Enemy, Explosion, ExplosionTimer, FromEnemy, FromPlayer, Laser, MainMenu, Movable, Player,
    ScoreBoardUI, SpriteSize, Velocity,
};
use directories::ProjectDirs;
use enemy::EnemyPlugin;
use player::PlayerPlugin;

mod components;
mod enemy;
mod player;

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144., 75.);
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_UPGRADE: &str = "laser_green.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);
const PLAYER_MAX_LASERS: usize = 10;

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.);

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize = 16;

const SPRITE_SCALE: f32 = 0.5;
const BASE_SPEED: f32 = 600.0;

const LASER_UPGRADE_SCORE: u32 = 50;

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
    player_laser_upgrade: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion_layout: Handle<TextureAtlasLayout>,
    explosion_texture: Handle<Image>,
}

#[derive(Resource, Deref, DerefMut)]
struct Score(u32);

#[derive(Resource, Deref, DerefMut)]
struct HighScore(u32);

#[derive(Resource, Deref, DerefMut)]
struct EnemyCount(u32);

#[derive(Resource, Deref, DerefMut)]
struct MaxEnemies(u32);

#[derive(Resource, Deref, DerefMut)]
struct LaserUpgrage(bool);

#[derive(Resource, Deref)]
struct HighScorePath(PathBuf);

fn get_high_score_path() -> io::Result<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "balestech", "rust_invaders") {
        let data_dir = proj_dirs.data_local_dir();
        fs::create_dir_all(data_dir)?;
        return Ok(data_dir.join("high_score.txt"));
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Could not determine data directory",
    ))
}

fn main() {
    let high_score_path = get_high_score_path().unwrap_or_default();
    let high_score: u32 = fs::read_to_string(&high_score_path)
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .insert_resource(HighScore(high_score))
        .insert_resource(Score(0))
        .insert_resource(EnemyCount(0))
        .insert_resource(MaxEnemies(3))
        .insert_resource(LaserUpgrage(false))
        .insert_resource(HighScorePath(high_score_path))
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
        .add_systems(
            Update,
            update_scoreboard.run_if(in_state(GameState::Playing)),
        )
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
    high_score: Res<HighScore>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Text::new(format!(
            "New Game [enter]\n\n\nmove: [a] & [d]\nshoot: [up-arrow]\n\n\nHigh Score: {}",
            **high_score
        )),
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
    let explosion_texture_handle = asset_server.load(EXPLOSION_SHEET);
    let explosion_texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::new(64, 64), 4, 4, None, None);
    let explosion_layout = texture_atlases.add(explosion_texture_atlas);

    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        player_laser_upgrade: asset_server.load(PLAYER_LASER_UPGRADE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion_layout,
        explosion_texture: explosion_texture_handle,
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
    mut max_enemies: ResMut<MaxEnemies>,
    mut enemy_count: ResMut<EnemyCount>,
    mut laser_velocity_upgrade: ResMut<LaserUpgrage>,
    explosion_query: Query<(), With<Explosion>>,
    enemy_query: Query<Entity, With<Enemy>>,
    score: Res<Score>,
    mut high_score: ResMut<HighScore>,
    high_score_path: Res<HighScorePath>,
) {
    // reset enemies & upgrades
    **max_enemies = 3;
    **laser_velocity_upgrade = false;
    for entity in &enemy_query {
        commands.entity(entity).despawn();
        **enemy_count -= 1;
    }

    // wait for explosions to finish
    if explosion_query.iter().len() == 0 {
        // check for new high score
        if **score > **high_score {
            **high_score = **score;
            let _ = fs::write(&**high_score_path, format!("{}", **high_score));
        }

        commands.spawn((
            Text::new(format!(
                "You Died!\nGame Over\n\nrestart [enter]\n\n\nHigh Score: {}",
                **high_score
            )),
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
    mut laser_velocity_upgrade: ResMut<LaserUpgrage>,
    mut max_enemies: ResMut<MaxEnemies>,
    score_root: Single<Entity, (With<ScoreBoardUI>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*score_root, 1) = score.to_string();

    if **score == 5 {
        **max_enemies = 10;
    }
    if **score == LASER_UPGRADE_SCORE {
        **laser_velocity_upgrade = true;
    }
}

fn movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut enemy_count: ResMut<EnemyCount>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
    enemy_query: Query<&Enemy>,
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
                if enemy_query.get(entity).is_ok() {
                    **enemy_count -= 1;
                }
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
