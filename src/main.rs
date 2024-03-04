use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa::Off)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Nevidita".into(),
                        name: Some("nevidita.app".into()),
                        present_mode: PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, (startup, spawn_map, spawn_unit).chain())
        .insert_resource(CurrentMap::default());

    #[cfg(debug_assertions)]
    {
        app.add_plugins(LogDiagnosticsPlugin::default())
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(WorldInspectorPlugin::new())
            .add_plugins(ResourceInspectorPlugin::<CurrentMap>::default());
    }

    app.run()
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_scale(Vec3::new(0.5, 0.5, 0.5)),
        ..default()
    });
}

#[derive(Resource, Reflect)]
pub struct CurrentMap {
    height: u32,
    width: u32,
    fields: Vec<Field>,
}

impl Default for CurrentMap {
    fn default() -> Self {
        Self {
            height: 10,
            width: 10,
            fields: vec![Field::Passable; 100],
        }
    }
}

impl CurrentMap {
    pub fn get(&self, pos: UVec2) -> Field {
        if pos.y > self.height && pos.x > self.width {
            panic!("tried to read map field with out of bound coordinate!");
        }

        self.fields[(pos.x + (pos.y * self.height)) as usize]
    }
}

#[derive(Reflect, Default, Clone, Copy)]
pub enum Field {
    #[default]
    Passable = 1,
    Impassable = 2,
}

pub mod flags {
    use super::*;

    /// FLag struct for enemy units
    #[derive(Component)]
    pub struct Enemy;

    /// Flag struct For allied units
    #[derive(Component)]
    pub struct Ally;
}

fn spawn_map(mut commands: Commands, map: Res<CurrentMap>, asset_server: ResMut<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize {
        x: map.width,
        y: map.height,
    };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map.width {
        for y in 0..map.height {
            let tile_pos = TilePos { x, y };
            let index = TileTextureIndex(match map.get(UVec2::new(x, y)) {
                Field::Passable => (x % 2 + y % 2) % 2,
                Field::Impassable => 2,
            });
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: index,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}

fn spawn_unit(
    mut commands: Commands,
    tile_map: Query<Entity, With<TileStorage>>,
    asset_server: ResMut<AssetServer>,
) {
    let guardian: Handle<Image> = asset_server.load("shield.png");
    let fighter: Handle<Image> = asset_server.load("sword.png");
    let striker: Handle<Image> = asset_server.load("bow.png");

    let guardian_enemy: Handle<Image> = asset_server.load("shield-enemy.png");
    let fighter_enemy: Handle<Image> = asset_server.load("sword-enemy.png");
    let striker_enemy: Handle<Image> = asset_server.load("bow-enemy.png");

    let map = tile_map.single();

    // spawn allied units
    commands
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            texture: striker,
            ..default()
        })
        .insert((Operator::STRIKER, flags::Ally))
        .set_parent(map)
        .insert(Name::new("STRIKER"))
        .commands()
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(4.0 * 16.0, 4.0 * 16.0, 1.0),
            texture: fighter,
            ..default()
        })
        .insert((Operator::FIGHTER, flags::Ally))
        .insert(Name::new("FIGHTER"))
        .set_parent(map)
        .commands()
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(32.0, 32.0, 1.0),
            texture: guardian,
            ..default()
        })
        .insert((Operator::GUARDIAN, flags::Ally))
        .insert(Name::new("GUARDIAN"))
        .set_parent(map);

    commands
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(9.0 * 16.0, 9.0 * 16.0, 1.0),
            texture: striker_enemy,
            ..default()
        })
        .insert((Operator::STRIKER, flags::Enemy))
        .insert(Name::new("STRIKER ENEMY"))
        .set_parent(map)
        .commands()
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(5.0 * 16.0, 5.0 * 16.0, 1.0),
            texture: fighter_enemy,
            ..default()
        })
        .insert((Operator::FIGHTER, flags::Enemy))
        .insert(Name::new("FIGHTER ENEMY"))
        .set_parent(map)
        .commands()
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(7.0 * 16.0, 7.0 * 16.0, 1.0),
            texture: guardian_enemy,
            ..default()
        })
        .insert((Operator::GUARDIAN, flags::Enemy))
        .insert(Name::new("GUARDIAN ENEMY"))
        .set_parent(map);
}

#[derive(Component)]
struct Operator {
    hp: f32,
    def: f32,
    atk: f32,
    mov: f32,
    rag: f32,
}

impl Operator {
    const GUARDIAN: Self = Self {
        hp: 200.0,
        def: 0.5,
        atk: 50.0,
        mov: 16.0,
        rag: 16.0,
    };

    const FIGHTER: Self = Self {
        hp: 150.0,
        def: 0.1,
        atk: 100.0,
        mov: 48.0,
        rag: 16.0,
    };

    const STRIKER: Self = Self {
        hp: 100.0,
        def: 0.0,
        atk: 75.0,
        mov: 32.0,
        rag: 64.0,
    };
}
