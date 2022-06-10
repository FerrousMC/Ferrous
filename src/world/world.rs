use std::collections::HashMap;
use crate::world::blocks::BlockState;

pub struct Dimension {
    chunks: HashMap<(i32, i32), Chunk>, // no idea if this is okay, rip ram
    generator: ChunkGenerator // todo, each dimension should have its own generator
}
impl Dimension {
    //
}

pub struct Chunk {
    xPos: i64,
    zPos: i64,
    sections: [ChunkSection; 24] // -64 to 319
}
impl Chunk {
    // Returns mutable reference to the ChunkSection at the index. Index is from -4 to 19 (in 1.18.2)
    pub fn section(&mut self, index: i32) -> &mut ChunkSection {
        &mut self.sections[(index + 4) as usize]
    }

    pub fn update(&mut self) {}
}

// 16x16x16 set of blocks. Chunks consist of several of these
pub struct ChunkSection {
    blocks: [[[BlockState; 16]; 16]; 16]
}
impl ChunkSection {
    // Returns a reference to the BlockState at the provided coords
    pub fn get_blockstate(&self, x: usize, y: usize, z: usize) -> &BlockState {
        &self.blocks[x][y][z]
    }

    // Returns a mutable reference to the BlockState at the provided coords
    pub fn get_blockstate_mut(&mut self, x: usize, y: usize, z: usize) -> &mut BlockState {
        &mut self.blocks[x][y][z]
    }

    // Sets the given BlockState to the given coords in the chunk section
    pub fn set_blockstate(&mut self, new_state: BlockState, x: usize, y: usize, z: usize) {
        self.blocks[x][y][z] = new_state;
    }
}

pub struct ChunkGenerator {
    // options for generation
    // ...
    gen_queue: Vec<(i32, i32)>
}