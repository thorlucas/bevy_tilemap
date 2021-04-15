use crate::{
    chunk::{SimpleTile, TileTrait},
    lib::*,
};

/// Common methods for layers in a chunk.
pub(super) trait Layer<T>: 'static
where
    T: TileTrait,
{
    /// Sets a raw tile for a layer at an index.
    fn set_tile(&mut self, index: usize, tile: T);

    /// Removes a tile for a layer at an index.
    fn remove_tile(&mut self, index: usize);

    /// Gets a tile by an index.
    fn get_tile(&self, index: usize) -> Option<&T>;

    /// Gets a tile with a mutable reference by an index.
    fn get_tile_mut(&mut self, index: usize) -> Option<&mut T>;

    /// Gets all the tile indices in the layer that exist.
    fn get_tile_indices(&self) -> Vec<usize>;

    /// Clears a layer of all sprites.
    fn clear(&mut self);

    /// Takes all the tiles in the layer and returns attributes for the renderer.
    fn tiles_to_attributes(&self, dimension: Dimension3) -> (Vec<f32>, Vec<[f32; 4]>);
}

/// A layer with dense sprite tiles.
///
/// The difference between a dense layer and a sparse layer is simply the
/// storage types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DenseLayer<T>
where
    T: TileTrait,
{
    /// A vector of all the tiles in the chunk.
    tiles: Vec<T>,
    /// A count of the tiles to keep track if layer is empty or not.
    tile_count: usize,
}

impl<T> Layer<T> for DenseLayer<T>
where
    T: TileTrait,
{
    fn set_tile(&mut self, index: usize, tile: T) {
        if let Some(inner_tile) = self.tiles.get_mut(index) {
            self.tile_count += 1;
            *inner_tile = tile;
        } else {
            warn!(
                "tile is out of bounds at index {} and can not be set",
                index
            );
        }
    }

    fn remove_tile(&mut self, index: usize) {
        if let Some(tile) = self.tiles.get_mut(index) {
            if self.tile_count != 0 {
                self.tile_count -= 1;
                tile.hide();
            }
        }
    }

    fn get_tile(&self, index: usize) -> Option<&T> {
        self.tiles
            .get(index)
            .and_then(|tile| if tile.is_hidden() { None } else { Some(tile) })
    }

    fn get_tile_mut(&mut self, index: usize) -> Option<&mut T> {
        self.tiles
            .get_mut(index)
            .and_then(|tile| if tile.is_hidden() { None } else { Some(tile) })
    }

    fn get_tile_indices(&self) -> Vec<usize> {
        let mut indices = Vec::with_capacity(self.tiles.len());
        for (index, tile) in self.tiles.iter().enumerate() {
            if !tile.is_hidden() {
                indices.push(index);
            }
        }
        indices.shrink_to_fit();
        indices
    }

    fn clear(&mut self) {
        self.tiles.clear();
    }

    fn tiles_to_attributes(&self, _dimension: Dimension3) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::chunk::raw_tile::dense_tiles_to_attributes(&self.tiles)
    }
}

impl<T> DenseLayer<T>
where
    T: TileTrait,
{
    /// Constructs a new dense layer with tiles.
    pub fn new(tiles: Vec<T>) -> DenseLayer<T> {
        DenseLayer {
            tiles,
            tile_count: 0,
        }
    }
}

/// A layer with sparse sprite tiles.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub(super) struct SparseLayer<T> {
    /// A map of all the tiles in the chunk.
    tiles: HashMap<usize, T>,
}

impl<T> Layer<T> for SparseLayer<T>
where
    T: TileTrait,
{
    fn set_tile(&mut self, index: usize, tile: T) {
        if tile.is_hidden() {
            self.tiles.remove(&index);
        }
        self.tiles.insert(index, tile);
    }

    fn remove_tile(&mut self, index: usize) {
        self.tiles.remove(&index);
    }

    fn get_tile(&self, index: usize) -> Option<&T> {
        self.tiles.get(&index)
    }

    fn get_tile_mut(&mut self, index: usize) -> Option<&mut T> {
        self.tiles.get_mut(&index)
    }

    fn get_tile_indices(&self) -> Vec<usize> {
        let mut indices = Vec::with_capacity(self.tiles.len());
        for index in self.tiles.keys() {
            indices.push(*index);
        }
        indices
    }

    fn clear(&mut self) {
        self.tiles.clear();
    }

    fn tiles_to_attributes(&self, dimension: Dimension3) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::chunk::raw_tile::sparse_tiles_to_attributes(dimension, &self.tiles)
    }
}

impl<T> SparseLayer<T> {
    /// Constructs a new sparse layer with a tile hashmap.
    pub fn new(tiles: HashMap<usize, T>) -> SparseLayer<T> {
        SparseLayer { tiles }
    }
}

/// Specifies which kind of layer to construct, either a dense or a sparse
/// sprite layer.
///
/// The difference between a dense and sparse layer is namely the storage kind.
/// A dense layer uses a vector and must fully contain tiles. This is ideal for
/// backgrounds. A sparse layer on the other hand uses a map with coordinates
/// to a tile. This is ideal for entities, objects or items.
///
/// It is highly recommended to adhere to the above principles to get the lowest
/// amount of byte usage.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum LayerKind {
    /// Specifies the tilemap to add a dense sprite layer.
    Dense,
    /// Specifies the tilemap to add a sparse sprite layer.
    Sparse,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
/// Inner enum used for storing either a dense or sparse layer.
pub(super) enum LayerKindInner<T>
where
    T: TileTrait,
{
    /// Inner dense layer storage.
    Dense(DenseLayer<T>),
    /// Inner sparse layer storage.
    Sparse(SparseLayer<T>),
}

impl<T> AsRef<dyn Layer<T>> for LayerKindInner<T>
where
    T: TileTrait,
{
    fn as_ref(&self) -> &dyn Layer<T> {
        match self {
            LayerKindInner::Dense(s) => s,
            LayerKindInner::Sparse(s) => s,
        }
    }
}

impl<T> AsMut<dyn Layer<T>> for LayerKindInner<T>
where
    T: TileTrait,
{
    fn as_mut(&mut self) -> &mut dyn Layer<T> {
        match self {
            LayerKindInner::Dense(s) => s,
            LayerKindInner::Sparse(s) => s,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
/// A sprite layer which can either store a sparse or dense layer.
pub(super) struct SpriteLayer {
    /// Enum storage of the kind of layer.
    pub inner: LayerKindInner<SimpleTile>,
}
