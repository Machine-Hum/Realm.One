use amethyst::{
    core::transform::Transform,
    ecs::{Component, DenseVecStorage, FlaggedStorage, Entity},
};

extern crate tiled;
use std::{
    fs::File,
    io::{BufReader},
    path::Path,
    fs,
};

use log::info;
use crate::constants;
use crate::components::Orientation;
use crate::mech::{colision};
use stringreader::StringReader;

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum Layers {
    L1 = 0,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
}

pub struct Room {
    pub map: tiled::Map,
    pub xsize: usize,
    pub tile_ent: Vec<Entity>,
    pub update: bool,
    pub name: String,
}

impl Default for Room {
    fn default() -> Self { 
        let file_name =  "resources/maps/first.tmx".to_string();
        let file = File::open(&Path::new(&file_name)).unwrap();
        let reader = BufReader::new(file);
        let map =  tiled::parse_with_path(reader, &Path::new("resources/sprites/master16.tsx")).unwrap();
        
        Self { 
            xsize: map.layers[0].tiles[0].len() - 1,
            map,
            tile_ent: Vec::new(),
            update: true,
            name: file_name,
        }
    }
}

impl Room {
    pub fn new(file_name: String) -> Self {
        let file = File::open(&Path::new(&file_name)).unwrap();
        let reader = BufReader::new(file);
        let map =  tiled::parse_with_path(reader, &Path::new("resources/sprites/master16.tsx")).unwrap();

        Self {
            xsize: map.layers[0].tiles[0].len() - 1,
            map, 
            tile_ent: Vec::new(),
            update: true,
            name: file_name,
        }
    }

    pub fn change(&mut self, map_name: String) {
        let file = File::open(&Path::new(&map_name)).unwrap();
        let reader = BufReader::new(file);
        let map =  tiled::parse_with_path(reader, &Path::new("resources/sprites/master16.tsx")).unwrap();
        
        self.map = map;
        self.update = true;
    }
    
    // Convert world coordinates to tiled coordinates
    fn world_2_tiled(&self, (x, y): (i32, i32)) -> (i32, i32){
        (x, (self.map.height as i32 - 1) - y)
    }

    pub fn get_pos(pos: &Transform) -> (i32, i32){
         Room::px_2_world(pos.translation().data[0], pos.translation().data[1])
    }
    
    // Convert from pixel coordinates 
    pub fn px_2_world(x: f32, y:f32) -> (i32, i32){
        ((((x - constants::TILE_SIZE) / constants::TILE_SIZE) as i32),
         (((y - constants::TILE_SIZE) / constants::TILE_SIZE) as i32)
        )
    }

    // Check to see if the resulting position is inside the map
    pub fn allowed_move(&self, pos: &Transform, facing: &Orientation) -> bool {
        let adj: Adj = self.get_adj(pos);
        let (x, y) = Room::get_pos(pos);

        let north = (*facing == Orientation::North)
            && ((y >= (self.map.height as i32 - constants::TILE_PER_PLAYER as i32))
                || colision(&adj.n));
        
        let east = (*facing == Orientation::East)
            && ((x >= (self.map.width as i32 - constants::TILE_PER_PLAYER as i32))
                || colision(&adj.e));
        
        let south = (*facing == Orientation::South) && ((y == 0) || colision(&adj.s));
        
        let west = (*facing == Orientation::West) && ((x == 0) || colision(&adj.w));

        !north && !east && !south && !west
    }
    
    fn get_prop(&self, (x, y): (i32, i32), (xoff, yoff): (i32, i32)) -> Option<tiled::Properties> {
        
        // Bottom left
        if (x == 0 && xoff <= -1) || (y == 0 && yoff <= -1) {
            return None;  
        }
        
        if x + xoff > (self.map.width as i32 - constants::TILE_PER_PLAYER as i32) {
            return None;
        }

        if y + yoff > (self.map.height as i32 - constants::TILE_PER_PLAYER as i32) {
            return None;
        }
        
        let (x1, y1): (i32, i32) = self.world_2_tiled((x + xoff, y + yoff));
        let tile = self.map.layers[Layers::L4 as usize].tiles[y1 as usize][x1 as usize];

        match self.map.get_tileset_by_gid(tile.gid) {
            Some(thing) => {
                Some(thing.tiles[tile.gid as usize].properties.clone())
            },
            None => None,
        }
    }
    
    pub fn get_adj(&self, pos: &Transform) -> Adj {
        let (x, y): (i32, i32) = Room::get_pos(pos);
        
        Adj{
            cur: self.get_prop((x,y),(0,0)),
            n:   self.get_prop((x,y),(0,constants::TILE_PER_PLAYER as i32)),
            e:   self.get_prop((x,y),(constants::TILE_PER_PLAYER as i32,0)),
            s:   self.get_prop((x,y),(0, -constants::TILE_PER_PLAYER as i32)),
            w:   self.get_prop((x,y),(-constants::TILE_PER_PLAYER as i32,0)),
        }
    }
}

pub struct Adj {
    pub cur: Option<tiled::Properties>,
    pub n: Option<tiled::Properties>,
    pub e: Option<tiled::Properties>,
    pub s: Option<tiled::Properties>,
    pub w: Option<tiled::Properties>,
}

impl Component for TilePosition{
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

pub struct TilePosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub gid: usize,
}

impl TilePosition {
    pub fn new(x: usize, y:usize, z: usize, gid: usize) -> Self {
        Self {
            x,
            y,
            z,
            gid,
        }
    }

    pub fn to_trans(&mut self) -> Transform {
        let mut transform = Transform::default();
        transform.set_translation_xyz((self.x as f32 * constants::TILE_SIZE) as f32 + 8.0, 
                                      (self.y as f32 * constants::TILE_SIZE) as f32 + 8.0, 
                                       self.z as f32 * 0.1
                                     );
        transform
    }
}
