use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::Rng;

use crate::{
    ENEMY_LASER_SIZE, ENEMY_SIZE, EnemyCount, GameTextures, MaxEnemies, SPRITE_SCALE, WinSize,
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
    max_enemies: Res<MaxEnemies>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
) {
    if **enemy_count < **max_enemies {
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
            .insert(Movable { auto_despawn: true })
            .insert(Enemy);
        **enemy_count += 1;
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

        let mut spawn_laser = |x_offset: f32| {
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

        spawn_laser(x_offset);
        spawn_laser(-x_offset);
    }
}

fn enemy_move(win_size: Res<WinSize>, mut query: Query<(&mut Velocity, &Transform), With<Enemy>>) {
    for (mut velocity, transform) in &mut query {
        let mut rng = rand::rng();
        let x = rng.random_range(-0.02..=0.02);
        let y = rng.random_range(-0.02..=0.02);

        velocity.x += x;
        velocity.y += y;

        let translation = transform.translation;
        if translation.x < -win_size.w / 2. - 50. {
            velocity.x = 0.3;
        }
        if translation.x > win_size.w / 2. + 50. {
            velocity.x = -0.3;
        }
        if translation.y < -win_size.h / 2. + 200. {
            velocity.y = 0.3;
        }
        if translation.y > win_size.h / 2. + 50. {
            velocity.y = -0.3;
        }
    }
}
