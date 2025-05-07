use bevy::prelude::*;

use crate::{
    GameState, GameTextures, PLAYER_LASER_SIZE, PLAYER_MAX_LASERS, PLAYER_SIZE, SPRITE_SCALE,
    WinSize,
    components::{FromPlayer, Laser, Movable, Player, SpriteSize, Velocity},
};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), player_spawn)
            .add_systems(Update, player_input)
            .add_systems(Update, player_fire);
    }
}

fn player_spawn(mut commands: Commands, game_textures: Res<GameTextures>, win_size: Res<WinSize>) {
    let bottom = -win_size.h / 2.0;
    commands
        .spawn((
            Sprite::from_image(game_textures.player.clone()),
            Transform {
                translation: Vec3::new(0., bottom + PLAYER_SIZE.1 / 2. * SPRITE_SCALE + 5., 10.),
                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                ..Default::default()
            },
        ))
        .insert(Player)
        .insert(SpriteSize::from(PLAYER_SIZE))
        .insert(Movable {
            auto_despawn: false,
        })
        .insert(Velocity { x: 0.0, y: 0.0 });
}

fn player_input(
    input: Res<ButtonInput<KeyCode>>,
    win_size: Res<WinSize>,
    mut query: Query<(&mut Velocity, &Transform), With<Player>>,
) {
    if let Ok((mut velocity, transform)) = query.single_mut() {
        let x = if input.pressed(KeyCode::KeyA) {
            -1.0
        } else if input.pressed(KeyCode::KeyD) {
            1.0
        } else {
            0.0
        };

        let translation = transform.translation;
        if translation.x < -win_size.w / 2. + PLAYER_SIZE.1 / 2. && x < 0.0 {
            velocity.x = 0.0;
            return;
        }
        if translation.x > win_size.w / 2. - PLAYER_SIZE.1 / 2. && x > 0.0 {
            velocity.x = 0.0;
            return;
        }

        velocity.x = x;
    }
}

fn player_fire(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    game_textures: Res<GameTextures>,
    query: Query<&Transform, With<Player>>,
    player_laser_query: Query<(), (With<Laser>, With<FromPlayer>)>,
) {
    if let Ok(player_tf) = query.single() {
        if input.just_pressed(KeyCode::ArrowUp)
            && player_laser_query.iter().len() < PLAYER_MAX_LASERS
        {
            let (x, y) = (player_tf.translation.x, player_tf.translation.y);
            let x_offset = PLAYER_SIZE.0 / 2. * SPRITE_SCALE - 5.;

            let mut spawn_lazer = |x_offset: f32| {
                commands
                    .spawn((
                        Sprite::from_image(game_textures.player_laser.clone()),
                        Transform {
                            translation: Vec3::new(x + x_offset, y + 15., 1.0),
                            scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.0),
                            ..Default::default()
                        },
                    ))
                    .insert(Laser)
                    .insert(FromPlayer)
                    .insert(SpriteSize::from(PLAYER_LASER_SIZE))
                    .insert(Movable { auto_despawn: true })
                    .insert(Velocity { x: 0.0, y: 1.0 });
            };

            spawn_lazer(x_offset);
            spawn_lazer(-x_offset);
        }
    }
}
