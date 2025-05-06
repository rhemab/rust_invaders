use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::Rng;

use crate::{
    ENEMY_LASER_SIZE, ENEMY_SIZE, EnemyCount, GameTextures, MAX_ENEMIES, SPRITE_SCALE, WinSize,
    components::{Enemy, FromEnemy, Laser, Movable, SpriteSize, Velocity},
};

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            enemy_spawn.run_if(on_timer(Duration::from_secs_f64(1.0))),
        )
        .add_systems(Update, enemy_move)
        .add_systems(
            Update,
            enemy_fire.run_if(on_timer(Duration::from_secs_f64(1.0))),
        );
    }
}

fn enemy_spawn(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
) {
    if enemy_count.0 < MAX_ENEMIES {
        let mut rng = rand::rng();
        let w_span = win_size.w / 2.0 - 100.0;
        let h_span = win_size.h / 2.0 - 100.0;
        let x = rng.random_range(-w_span..w_span);
        let y = rng.random_range(-h_span..h_span);
        commands
            .spawn((
                Sprite::from_image(game_textures.enemy.clone()),
                Transform {
                    translation: Vec3::new(x, y, 10.0),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    ..Default::default()
                },
            ))
            .insert(SpriteSize::from(ENEMY_SIZE))
            .insert(Velocity { x: 0.0, y: 0.0 })
            .insert(Movable {
                auto_despawn: false,
            })
            .insert(Enemy);
        enemy_count.0 += 1;
    }
}

fn enemy_fire(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<&Transform, With<Enemy>>,
) {
    for enemy_tf in &query {
        let (x, y) = (enemy_tf.translation.x, enemy_tf.translation.y);
        let x_offset = ENEMY_SIZE.0 / 2. * SPRITE_SCALE - 25.;

        let mut spawn_lazer = |x_offset: f32| {
            commands
                .spawn((
                    Sprite::from_image(game_textures.enemy_laser.clone()),
                    Transform {
                        translation: Vec3::new(x + x_offset, y, 1.0),
                        scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.0),
                        ..Default::default()
                    },
                ))
                .insert(Laser)
                .insert(FromEnemy)
                .insert(SpriteSize::from(ENEMY_LASER_SIZE))
                .insert(Movable { auto_despawn: true })
                .insert(Velocity { x: 0.0, y: -1.0 });
        };

        spawn_lazer(x_offset);
        spawn_lazer(-x_offset);
    }
}

fn enemy_move(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut enemy_count: ResMut<EnemyCount>,
    mut query: Query<(Entity, &mut Velocity, &Transform), With<Enemy>>,
) {
    for (entity, mut velocity, transform) in &mut query {
        let mut rng = rand::rng();
        let x = rng.random_range(-0.05..=0.05);
        let y = rng.random_range(-0.05..=0.05);
        velocity.x += x;
        velocity.y += y;

        let translation = transform.translation;
        let margin = 200.0;
        // if translation.x < -win_size.w / 2. {
        //     velocity.x += 1.0;
        // }
        // if translation.x > win_size.w / 2. {
        //     velocity.x += -1.0;
        // }
        // if translation.y < -win_size.h / 2. {
        //     velocity.y += 1.0;
        // }
        // if translation.y > win_size.h / 2.0 {
        //     velocity.y += -1.0;
        // }
        if translation.y > win_size.h / 2. + margin
            || translation.y < -win_size.h / 2. - margin
            || translation.x > win_size.w / 2. + margin
            || translation.x < -win_size.w / 2. - margin
        {
            commands.entity(entity).despawn();
            enemy_count.0 -= 1;
        }
    }
}
