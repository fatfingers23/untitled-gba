use serde::Deserialize;

const LEVELS: &[&str] = &["level_1", "level_2"];

// Work-around to allow console output from build script.
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    p!("Running build script");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    for &level in LEVELS {
        tiled_export::export_level(&out_dir, level).expect("Failed to export level");
    }
}

mod tiled_export {
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufReader, BufWriter, Write};

    const COLLISION_TILE: i32 = 1;
    const KILL_TILE: i32 = 2;
    const WIN_TILE: i32 = 4;

    fn get_tile_info(file: String) -> String {
        let file = File::open(file).expect("Failed to open file");

        let reader = BufReader::new(file);

        let tilemap: TiledTilemap = serde_json::from_reader(reader).expect("Failed to parse file");

        let tile_data: HashMap<_, _> = tilemap
            .tiles
            .unwrap_or(vec![])
            .iter()
            .map(|tile| {
                (
                    tile.id,
                    match tile.tile_type.as_str() {
                        "Collision" => COLLISION_TILE,
                        "Kill" => KILL_TILE,
                        "Win" => WIN_TILE,
                        _ => 0,
                    },
                )
            })
            .collect();

        (0..tilemap.tilecount)
            .map(|id| *tile_data.get(&id).unwrap_or(&0))
            .map(|tile_type| tile_type.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn export_level(out_dir: &str, level_file: &str) -> std::io::Result<()> {
        let background_tile_info = get_tile_info(format!(
            "map/{level_file}/{level_file}_background_tile_set.json"
        ));

        let foreground_tile_info = get_tile_info(format!(
            "map/{level_file}/{level_file}_foreground_tile_set.json"
        ));

        let filename = format!("map/{level_file}/{level_file}_map.json");
        println!("cargo:rerun-if-changed={filename}");
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let level: TiledLevel = serde_json::from_reader(reader)?;

        let output_file = File::create(format!("{out_dir}/{level_file}.rs"))?;
        let mut writer = BufWriter::new(output_file);

        let background_first_id = level
            .tilesets
            .iter()
            .filter(|tileset| tileset.source.contains("background"))
            .next()
            .unwrap()
            .firstgid;

        let foreground_first_id = level
            .tilesets
            .iter()
            .filter(|tileset| tileset.source.contains("foreground"))
            .next()
            .unwrap()
            .firstgid;

        let world_layer = level
            .layers
            .iter()
            .filter(|layer| layer.name == Layers::World)
            .next()
            .unwrap()
            .data
            .as_ref()
            .expect("Expected first layer to be a tile layer")
            .iter()
            .map(|id| get_map_id(*id, background_first_id).to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let world_objects_layer = level
            .layers
            .iter()
            .filter(|layer| layer.name == Layers::WorldObjects)
            .next()
            .unwrap()
            .data
            .as_ref()
            .expect("Expected second layer to be a tile layer")
            .iter()
            .map(|id| get_map_id(*id, foreground_first_id).to_string())
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(&mut writer, "const WIDTH: u32 = {};", level.width)?;
        writeln!(&mut writer, "const HEIGHT: u32 = {};", level.height)?;
        writeln!(
            &mut writer,
            "const FOREGROUND: &[u16] = &[{world_objects_layer}];"
        )?;
        writeln!(&mut writer, "const BACKGROUND: &[u16] = &[{world_layer}];")?;

        let objects_from_file = level
            .layers
            .iter()
            .filter(|layer| layer.name == Layers::Objects)
            .next()
            .unwrap()
            .objects
            .as_ref()
            .expect("Expected third layer to be an object layer")
            .iter();

        let objects = objects_from_file.map(|object| (&object.object_type, (object.x, object.y)));

        let mut snails = vec![];
        let mut slimes = vec![];
        let mut enemy_stops = vec![];
        let mut player_start = None;

        for (object_type, (x, y)) in objects {
            match object_type.as_str() {
                "Snail Spawn" => snails.push((x, y)),
                "Slime Spawn" => slimes.push((x, y)),
                "Player Start" => player_start = Some((x, y)),
                "Enemy Stop" => enemy_stops.push((x, y)),
                _ => panic!("Unknown object type {object_type}"),
            }
        }

        let player_start = player_start.expect("Need a start place for the player");

        let slimes_str = slimes
            .iter()
            .map(|slime| format!("({}, {})", slime.0, slime.1))
            .collect::<Vec<_>>()
            .join(", ");
        let snails_str = snails
            .iter()
            .map(|slime| format!("({}, {})", slime.0, slime.1))
            .collect::<Vec<_>>()
            .join(", ");
        let enemy_stop_str = enemy_stops
            .iter()
            .map(|enemy_stop| format!("({}, {})", enemy_stop.0, enemy_stop.1))
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(
            &mut writer,
            "const SNAILS: &[(i32, i32)] = &[{snails_str}];",
        )?;
        writeln!(
            &mut writer,
            "const SLIMES: &[(i32, i32)] = &[{slimes_str}];",
        )?;
        writeln!(
            &mut writer,
            "const ENEMY_STOPS: &[(i32, i32)] = &[{enemy_stop_str}];",
        )?;
        writeln!(
            &mut writer,
            "const START_POS: (i32, i32) = ({}, {});",
            player_start.0, player_start.1
        )?;
        writeln!(
            &mut writer,
            "pub const BACKGROUND_LEVEL_TILE_DATA: &[u32] = &[{background_tile_info}];"
        )?;

        writeln!(
            &mut writer,
            "pub const FOREGROUND_LEVEL_TILE_DATA: &[u32] = &[{foreground_tile_info}];"
        )?;

        writeln!(
            &mut writer,
            r#"
            use crate::level::Level;
            use agb::fixnum::Vector2D;

            agb::include_background_gfx!(
                games, "2ce8f4",
                {level_file}_background  => 16 deduplicate "gfx/tileSets/{level_file}/{level_file}_background.png",
                {level_file}_foreground => 16  deduplicate "gfx/tileSets/{level_file}/{level_file}_foreground.png"
            );
            pub const fn get_level() -> Level<'static> {{
                Level {{
                    background: BACKGROUND,
                    foreground: FOREGROUND,
                    dimensions: Vector2D {{x: WIDTH, y: HEIGHT}},
                    background_collision: BACKGROUND_LEVEL_TILE_DATA,
                    foreground_collision: FOREGROUND_LEVEL_TILE_DATA,
                    
                    enemy_stops: ENEMY_STOPS,
                    slimes: SLIMES,
                    snails: SNAILS,
                    start_pos: START_POS,
                    background_tile_set: games::{level_file}_background.tiles,
                    background_tile_settings: games::{level_file}_background.tile_settings,
                    foreground_tile_set: games::{level_file}_foreground.tiles,
                    foreground_tile_settings: games::{level_file}_foreground.tile_settings,
                }}
            }}
            "#
        )?;

        Ok(())
    }

    fn get_map_id(id: i32, offset: i32) -> i32 {
        match offset {
            1 => match id {
                0 => 0,
                i => i - 1,
            },
            _ => match id {
                0 => 0,
                i => (offset - i).abs(),
            },
        }
    }

    #[derive(Deserialize)]
    struct TiledLevel {
        layers: Vec<TiledLayer>,
        width: i32,
        height: i32,
        tilesets: Vec<TileSet>,
    }

    #[derive(Deserialize, PartialEq)]
    pub enum Layers {
        World,
        WorldObjects,
        Objects,
    }

    #[derive(Deserialize)]
    struct TiledLayer {
        name: Layers,
        data: Option<Vec<i32>>,
        objects: Option<Vec<TiledObject>>,
    }

    #[derive(Deserialize)]
    struct TiledObject {
        #[serde(rename = "type")]
        object_type: String,
        #[serde(deserialize_with = "float_to_i32")]
        x: i32,
        #[serde(deserialize_with = "float_to_i32")]
        y: i32,
    }

    #[derive(Deserialize)]
    struct TiledTilemap {
        tiles: Option<Vec<TiledTile>>,
        tilecount: i32,
    }

    #[derive(Deserialize)]
    struct TiledTile {
        id: i32,
        #[serde(rename = "type")]
        tile_type: String,
    }
    #[derive(Deserialize)]
    struct TileSet {
        pub firstgid: i32,
        pub source: String,
    }

    fn float_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let f = f32::deserialize(deserializer)?;
        Ok(f as i32)
    }
}
