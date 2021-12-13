use libp2p::{
    floodsub::{Floodsub, FloodsubEvent},
    kad::{
        record::{
            store::{Error, MemoryStore},
            Key,
        },
        AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryId, QueryResult,
        Quorum, Record,
    },
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};

use log::{debug, error};

use log::info;
use std::collections::HashSet;

// We create a custom network behaviour that combines floodsub and mDNS.
// The derive generates a delegating `NetworkBehaviour` impl which in turn
// requires the implementations of `NetworkBehaviourEventProcess` for
// the events of each behaviour.
#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct MyBehaviour {
    floodsub: Floodsub,
    kademlia: Kademlia<MemoryStore>,
    mdns: Mdns,
}

impl MyBehaviour {
    pub fn new(floodsub: Floodsub, kademlia: Kademlia<MemoryStore>, mdns: Mdns) -> Self {
        MyBehaviour {
            floodsub,
            kademlia,
            mdns,
        }
    }

    pub fn floodsub_mut(&mut self) -> &mut Floodsub {
        &mut self.floodsub
    }

    pub async fn lookup_blob(&mut self, hash: String, tx: tokio::sync::mpsc::Sender<String>) {
        //let num = std::num::NonZeroUsize::new(2).ok_or(Error::ValueTooLarge)?;
        //Ok(self.kademlia.get_record(&Key::new(&hash), Quorum::N(num)))
        // TODO
        match tx.send(String::from("coming soon")).await {
            Ok(_) => debug!("response for lookup_blob sent"),
            Err(_) => error!("failed to send response"),
        }
    }

    pub async fn list_peers(&mut self, tx: tokio::sync::mpsc::Sender<String>) {
        let mut peers = String::new();
        match get_peers(&mut self.mdns) {
            Ok(val) => peers = val,
            Err(e) => error!("failed to get peers connected: {:?}", e),
        }

        match tx.send(peers).await {
            Ok(_) => debug!("response for list_peers sent"),
            Err(_) => error!("failed to send response"),
        }
    }

    pub async fn list_peers_cmd(&mut self) {
        match get_peers(&mut self.mdns) {
            Ok(val) => println!("Peers are : {}", val),
            Err(e) => error!("failed to get peers connected: {:?}", e),
        }
    }

    pub fn advertise_blob(&mut self, hash: String, value: Vec<u8>) -> Result<QueryId, Error> {
        let num = std::num::NonZeroUsize::new(2).ok_or(Error::ValueTooLarge)?;
        self.kademlia
            .put_record(Record::new(Key::new(&hash), value), Quorum::N(num))
    }
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for MyBehaviour {
    // Called when `floodsub` produces an event.
    fn inject_event(&mut self, message: FloodsubEvent) {
        if let FloodsubEvent::Message(message) = message {
            info!(
                "Received: '{:?}' from {:?}",
                String::from_utf8_lossy(&message.data),
                message.source
            );
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for MyBehaviour {
    // Called when `mdns` produces an event.
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, multiaddr) in list {
                    self.floodsub.add_node_to_partial_view(peer);
                    self.kademlia.add_address(&peer, multiaddr);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, multiaddr) in list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                        self.kademlia.remove_address(&peer, &multiaddr);
                    }
                }
            }
        }
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for MyBehaviour {
    // Called when `kademlia` produces an event.
    fn inject_event(&mut self, message: KademliaEvent) {
        match message {
            KademliaEvent::OutboundQueryCompleted { result, .. } => match result {
                QueryResult::GetProviders(Ok(ok)) => {
                    for peer in ok.providers {
                        println!(
                            "Peer {:?} provides key {:?}",
                            peer,
                            std::str::from_utf8(ok.key.as_ref()).unwrap()
                        );
                    }
                }
                QueryResult::GetProviders(Err(err)) => {
                    eprintln!("Failed to get providers: {:?}", err);
                }
                QueryResult::GetRecord(Ok(ok)) => {
                    for PeerRecord {
                        record: Record { key, value, .. },
                        ..
                    } in ok.records
                    {
                        println!(
                            "Got record {:?} {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            std::str::from_utf8(&value).unwrap(),
                        );
                    }
                }
                QueryResult::GetRecord(Err(err)) => {
                    eprintln!("Failed to get record: {:?}", err);
                }
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    println!(
                        "Successfully put record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::PutRecord(Err(err)) => {
                    eprintln!("Failed to put record: {:?}", err);
                }
                QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                    println!(
                        "Successfully put provider record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::StartProviding(Err(err)) => {
                    eprintln!("Failed to put provider record: {:?}", err);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

pub fn get_peers(mdns: &mut Mdns) -> Result<String, Error> {
    let nodes = mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    let connected_peers = itertools::join(&unique_peers, ", ");
    Ok(connected_peers)
}
