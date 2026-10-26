use crate::World;
use engine_core::Mesh;
use std::{
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread,
};
pub struct ChunkData {
    pub key: (i32, i32),
    pub lod: u8,
    pub mesh: Mesh,
}
pub struct Streamer {
    sender: Option<SyncSender<((i32, i32), u8)>>,
    receiver: Option<Receiver<ChunkData>>,
    recycle: SyncSender<Vec<engine_core::Vertex>>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Streamer {
    pub fn new(world: Arc<World>) -> Self {
        let (tx, rx) = mpsc::sync_channel::<((i32, i32), u8)>(32);
        let (out_tx, out_rx) = mpsc::sync_channel(4);
        let (recycle, recycled) = mpsc::sync_channel(8);
        let worker = thread::spawn(move || {
            while let Ok((key, lod)) = rx.recv() {
                let vertices = recycled.try_recv().unwrap_or_default();
                let mesh = world.chunk_reusing(key, lod, vertices);
                if out_tx.send(ChunkData { key, lod, mesh }).is_err() {
                    break;
                }
            }
        });
        Self {
            sender: Some(tx),
            receiver: Some(out_rx),
            worker: Some(worker),
            recycle,
        }
    }
    pub fn request(&self, key: (i32, i32), lod: u8) -> bool {
        self.sender
            .as_ref()
            .is_some_and(|s| s.try_send((key, lod)).is_ok())
    }
    pub fn recycle(&self, mesh: Mesh) {
        let _ = self.recycle.try_send(mesh.vertices);
    }
    pub fn poll(&self) -> Option<ChunkData> {
        self.receiver.as_ref()?.try_recv().ok()
    }
}
impl Drop for Streamer {
    fn drop(&mut self) {
        self.sender.take();
        self.receiver.take();
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closing_a_full_job_queue_does_not_deadlock() {
        let stream = Streamer::new(Arc::new(World::new()));
        for x in -8..8 {
            for z in -8..8 {
                stream.request((x, z), 1);
            }
        }
        drop(stream);
    }
}
