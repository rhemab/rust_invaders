use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::Rng;

use crate::{
    ENEMY_SIZE, GameTextures, SPRITE_SCALE, WinSize,
    components::{Enemy, SpriteSize},
};

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            enemy_spawn.run_if(on_timer(Duration::from_secs_f64(0.5))),
        );
    }
}

fn enemy_spawn(mut commands: Commands, game_textures: Res<GameTextures>, win_size: Res<WinSize>) {
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
        .insert(Enemy);
}
