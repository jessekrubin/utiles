use crate::TileParent;

/// Iterator over parent tiles.
pub struct Parents<T>
where
    T: TileParent + Clone,
{
    pub current: Option<T>,
}

impl<T> Iterator for Parents<T>
where
    T: TileParent + Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_tile) = self.current {
            // Move to the next parent
            self.current = current_tile.parent(None);
            Some(current_tile)
        } else {
            None
        }
    }
}
