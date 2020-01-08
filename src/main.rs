use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    input::InputBundle,
    utils::application_root_dir,
    network::NetworkBundle,
};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::env; 
use log::info;

mod map;
mod key_bindings;
mod states;
mod components;
mod systems;
mod constants;
mod mech;
mod character_sprites;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let args: Vec<String> = env::args().collect();
    let mut rtn : amethyst::Result<()> = Ok(()); 
    let app_root = application_root_dir()?;
    let resources = app_root.join("resources");
    
    if args[1] == "client" {
        info!("Starting the client");
        rtn = client(resources);
    }

    else if args[1] == "server"{
        info!("Starting the server!");
        rtn = server(resources);
    }
    // else error out
    
    rtn
}

fn client(resources: std::path::PathBuf) -> amethyst::Result<()> {
    let display_config = resources.join("display_config.ron");
    let key_bindings_config_path = resources.join("bindings.ron");
    
    let input_bundle = InputBundle::<key_bindings::MovementBindingTypes>::new()
        .with_bindings_from_file(key_bindings_config_path)?;
    
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let network_bundle = NetworkBundle::<String>::new(socket);
     
    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        .with_bundle(input_bundle)? 
        .with_bundle(network_bundle)? 
        .with(systems::PlayerSystem, "player_system", &["input_system"]);


    let mut game = Application::new(
        resources, 
        states::GamePlayState{},
        game_data,
    )?;

    game.run();
    Ok(())
}

fn server(resources: std::path::PathBuf) -> amethyst::Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let network_bundle = NetworkBundle::<String>::new(socket);
     
    let game_data = GameDataBuilder::default()
        .with_bundle(network_bundle)?; 

    let mut game = Application::new(
        resources, 
        states::GamePlayState{},
        game_data,
    )?;

    game.run();
    Ok(())
}
