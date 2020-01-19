use amethyst::{
    renderer::SpriteRender,
    core::transform::Transform,
    prelude::*,
    ecs::{Component, DenseVecStorage, FlaggedStorage},
};
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub enum Orientation {
    South,
    West,
    East,
    North,
}

#[warn(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerAction {
    Nothing = 0,
    MoveN,
    MoveE,
    MoveS,
    MoveW,
}

pub struct PlayerList {
    pub list: Vec<PlayerInfo>,
}

impl Default for PlayerList {
    fn default() -> Self {
    Self{ list: Vec::new(), } 
    }
}

/// Server Size player components
#[warn(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInfo {
    pub id: u32,
    pub modified: bool, 
    pub act: PlayerAction,
    pub name: String,
    pub room: String,
    pub x: f32,          
    pub y: f32, 
    pub no: usize,      
    pub ea: usize, 
    pub so: usize,
    pub we: usize, 
}

/// Client Side player component
pub struct PlayerComponent {
    pub orientation: Orientation,
    pub n: SpriteRender,
    pub e: SpriteRender,
    pub s: SpriteRender,
    pub w: SpriteRender,
    pub last_movement_instant: Instant,
    pub trans: Transform,
    p: PlayerInfo,
}

impl PlayerComponent {
    pub fn new(p: PlayerInfo, sprites: &Vec<SpriteRender>) -> Self {
        let mut tr = Transform::default();
        tr.set_translation_xyz(p.x, p.y, 1.0);

        Self {
            n: sprites[p.no].clone(), 
            e: sprites[p.ea].clone(), 
            s: sprites[p.so].clone(), 
            w: sprites[p.we].clone(),
            orientation: Orientation::South,
            last_movement_instant: Instant::now(),
            trans: tr,
            p,
        }
    }
   
    pub fn get_orientated(&self) -> SpriteRender {
        match self.orientation {
            Orientation::North=> return self.n.clone(),
            Orientation::South=> return self.s.clone(),
            Orientation::East => return self.e.clone(),
            Orientation::West => return self.w.clone(),
        }
    }
}

impl Component for PlayerComponent {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
