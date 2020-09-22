use amethyst::{
    core::{SystemDesc, bundle::SystemBundle},
    ecs::{Read, Write, System, SystemData, World, DispatcherBuilder},
    shrev::{EventChannel, ReaderId}, 
    network::simulation::{NetworkSimulationEvent, TransportResource, NetworkSimulationTime},
    Result,
};

use log::{info, warn, error};
use crate::network::{Pack, Cmd, Dest};
use crate::resources::{LifeformList};
use crate::systems::server::{AuthEvent, LifeformEvent};
use std::net::{SocketAddr};

#[derive(Debug)]
pub struct TcpSystemBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for TcpSystemBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(
            TcpSystemDesc::default().build(world),
            "server_tcp_system",
            &[],
        );
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct TcpSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, TcpSystem> for TcpSystemDesc {
    fn build(self, world: &mut World) -> TcpSystem {
        <TcpSystem as System<'_>>::SystemData::setup(world);
        let net_reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();
        let event_reader = world
            .fetch_mut::<EventChannel<Pack>>()
            .register_reader();

        TcpSystem::new(net_reader, event_reader)
    }
}

pub struct TcpSystem {
    net_reader: ReaderId<NetworkSimulationEvent>,
    event_reader: ReaderId<Pack>,
    clients: Vec<SocketAddr>,
}

impl TcpSystem {
    pub fn new(
        net_reader: ReaderId<NetworkSimulationEvent>, 
        event_reader: ReaderId<Pack>, 
    ) -> Self 
    {
        Self { 
            net_reader,
            event_reader,
            clients: Vec::<SocketAddr>::new(),
        }
    }
}

impl<'a> System<'a> for TcpSystem {
    type SystemData = (
        Write<'a, EventChannel<Pack>>,
        Write<'a, EventChannel<LifeformEvent>>,
        Write<'a, EventChannel<AuthEvent>>,
        Write<'a, TransportResource>,
        Read<'a, NetworkSimulationTime>,
        Read<'a, EventChannel<NetworkSimulationEvent>>,
        Write<'a, LifeformList>,
        // Write<'a, EventChannel<WalletEvents>>,
    );

    fn run(&mut self, (mut in_packs, mut lf, mut auth, mut net, sim_time, channel, mut pl/*, mut wallet_events*/): Self::SystemData) {
        let mut packs = Vec::<Pack>::new();
        // First we get the Events
        for event in channel.read(&mut self.net_reader) {
            match event {
                NetworkSimulationEvent::Message(addr, payload) => {
                    info!("Package: {:?}", payload);
                    let mut pk = Pack::from_bin(payload.to_vec());
                    pk.dest = Dest::Ip(addr.clone());  // Update the client addr
                    packs.push(pk);
                }
                NetworkSimulationEvent::Connect(addr) => {
                    info!("New client connection: {}", addr);
                    self.clients.push(*addr);
                }
                NetworkSimulationEvent::Disconnect(addr) => {
                    info!("Client Disconnected: {}", addr);
                    self.clients.retain(|&x| x != *addr);
                   
                    match pl.get_from_ip(*addr) {
                        Some(player) => {
                            let id = player.id();
                            lf.single_write(LifeformEvent::RemovePlayer(id));
                            in_packs.single_write(Pack::new(Cmd::RemovePlayer(id), Dest::All)); 
                        },
                        None => warn!("Player disconnected that was not on the playerlist"),
                    }
                }
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                _ => {}
            }
        }
        
        // Then we process the Events
        for pack in packs {
            match &pack.cmd {
                Cmd::Connect(s) => auth.single_write(AuthEvent::Connect(s.to_string(), pack.ip().unwrap())),
                Cmd::Action(act) => {
                    if let Some(player) = pl.get_from_ip(pack.ip().unwrap()) {
                        lf.single_write(LifeformEvent::Action(act.clone(), player));
                    }
                    else {
                        warn!("Pack from someone not on the playerlist!");
                    }
                },
                Cmd::RemovePlayer(uid) => lf.single_write(LifeformEvent::RemovePlayer(*uid)),
                Cmd::NewItem(_item) => {
                    if let Some(_player) = pl.get_from_ip(pack.ip().unwrap()) {
                        // wallet_events.single_write();
                    }
                    else {
                        warn!("Pack from someone not on the playerlist!");
                    }
                }
                _ => (),
            }
        }

        // This is the new way!
        for _frame in sim_time.sim_frames_to_run() {
            for pack in in_packs.read(&mut self.event_reader) {
                match &pack.dest {
                    // Just send to one address 
                    Dest::Ip(addr) => {
                        // info!("Sending pack: {:?} to: {:?}", pack, addr);
                        net.send(*addr, &pack.to_bin());
                    },
                    // Broadcast message
                    Dest::All => {
                        for addr in &self.clients {
                            // info!("Sending pack: {:?} to: {:?}", pack, addr);
                            net.send(*addr, &pack.to_bin());
                        }
                    },
                    Dest::Room(name) => {
                        // Get all the ip's in the room
                        let ips = pl.ip_in_room(&name);
                        for ip in ips { 
                            // info!("Sending pack: {:?} to: {:?}", pack, ip);
                            net.send(ip, &pack.to_bin());
                        }
                    },
                    Dest::AllExcept(ip) => {
                        for addr in &self.clients {
                            if addr != ip {
                                // info!("Sending pack: {:?} to: {:?}", pack, addr);
                                net.send(*addr, &pack.to_bin());
                            }
                        }
                    }
                }
            }
        }
    }
}
