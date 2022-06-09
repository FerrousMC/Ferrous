pub struct Dimension {
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
    //
}
impl ChunkSection {
    //
}