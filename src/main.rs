use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    window::PrimaryWindow,
};
use components::{Enemy, FromPlayer, Laser, Movable, SpriteSize, Velocity};
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

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize = 16;

const SPRITE_SCALE: f32 = 0.5;
const BASE_SPEED: f32 = 600.0;

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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
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
        .add_systems(Update, movement)
        .add_systems(Update, player_laser_hit_enemy)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<&Window, With<PrimaryWindow>>,
) {
    commands.spawn(Camera2d);

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
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
    for (laser_entity, laser_tf, laser_size) in &laser_query {
        let laser_scale = Vec2::from(laser_tf.scale.xy());

        for (enemy_entity, enemy_tf, enemy_size) in &enemy_query {
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
                commands.entity(enemy_entity).despawn();
                commands.entity(laser_entity).despawn();
            }
        }
    }
}
