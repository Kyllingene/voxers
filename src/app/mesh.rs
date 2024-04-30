use std::collections::HashMap;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::thread;

use super::chunk::Chunk;
use crate::renderer::mesh::Mesh;

mod fast;
mod greedy;

pub use fast::fast;
pub use greedy::greedy;

pub struct BgMesher {
    pub send: SyncSender<([i32; 3], ChunkRegion)>,
    pub recv: Receiver<([i32; 3], Mesh)>,
    pub full: bool,
    closed: bool,
}

impl BgMesher {
    pub const MESHER: fn([i32; 3], ChunkRegion) -> Mesh = greedy;

    pub fn new() -> Self {
        let (mesh_tx, recv) = sync_channel(1);
        let (send, mesh_rx) = sync_channel(0);

        thread::spawn(move || {
            while let Ok((pos, chunks)) = mesh_rx.recv() {
                let Ok(_) = mesh_tx.send((pos, Self::MESHER(pos, chunks))) else {
                    break;
                };
            }
        });

        Self {
            send,
            recv,
            full: false,
            closed: false,
        }
    }

    pub fn mesh(&mut self, [x, y, z]: [i32; 3], chunks: &HashMap<[i32; 3], Chunk>) -> bool {
        if self.closed || self.full {
            return false;
        }

        self.full = true;

        let region = ChunkRegion {
            center: chunks.get(&[x, y, z]).unwrap().clone(),

            left: chunks.get(&[x - 1, y, z]).cloned(),
            right: chunks.get(&[x + 1, y, z]).cloned(),
            top: chunks.get(&[x, y + 1, z]).cloned(),
            bot: chunks.get(&[x, y - 1, z]).cloned(),
            front: chunks.get(&[x, y, z - 1]).cloned(),
            back: chunks.get(&[x, y, z + 1]).cloned(),
        };

        self.closed = self.send.send(([x, y, z], region)).is_err();
        !self.closed
    }

    pub fn query<'a>(&'a mut self) -> Option<([i32; 3], Mesh)> {
        let data = self.recv.try_recv();
        if matches!(data, Err(TryRecvError::Disconnected)) {
            self.closed = true;
        } else if data.is_ok() {
            self.full = false;
        }
        data.ok()
    }
}

pub struct ChunkRegion {
    center: Chunk,

    left: Option<Chunk>,
    right: Option<Chunk>,
    top: Option<Chunk>,
    bot: Option<Chunk>,
    front: Option<Chunk>,
    back: Option<Chunk>,
}
