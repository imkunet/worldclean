use std::io::{Read, Seek};

use anvil_region::{position::RegionChunkPosition, region::Region};
use nbt::CompoundTag;

pub(crate) trait IntoRegionIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_ext_iter(self) -> Self::IntoIter;
}

impl<S: Read + Seek> IntoRegionIterator for Region<S> {
    type Item = <RegionIterator<S> as Iterator>::Item;
    type IntoIter = RegionIterator<S>;

    fn into_ext_iter(self) -> Self::IntoIter {
        RegionIterator {
            inner: self,
            current: 0,
        }
    }
}

pub struct RegionIterator<S: Read + Seek> {
    inner: Region<S>,
    current: usize,
}

impl<S: Read + Seek> Iterator for RegionIterator<S> {
    type Item = (CompoundTag, RegionChunkPosition);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == 1024 {
            return None;
        }

        let x = self.current % 32;
        let z = self.current / 32;

        self.current += 1;

        let pos = RegionChunkPosition::new(x as u8, z as u8);
        match self.inner.read_chunk(pos) {
            Ok(chunk) => Some((chunk, pos)),
            Err(_) => self.next(),
        }
    }
}
