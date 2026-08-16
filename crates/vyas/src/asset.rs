use std::{
    collections::{HashMap, HashSet, hash_map},
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    color::{Color, PaletteColor, Srgb},
    position::GridPosition,
    voxel::Voxel,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct VoxelAsset {
    pub grid: Grid,
    pub colors: Vec<PaletteColor>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Grid(HashMap<GridPosition, Voxel>);

impl Grid {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn into_inner(self) -> HashMap<GridPosition, Voxel> {
        self.0
    }
}

impl Serialize for Grid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.iter().collect::<Vec<_>>().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Grid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<(GridPosition, Voxel)>::deserialize(deserializer)
            .map(|entries| Self(entries.into_iter().collect()))
    }
}

impl Deref for Grid {
    type Target = HashMap<GridPosition, Voxel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HashMap<GridPosition, Voxel>> for Grid {
    fn from(grid: HashMap<GridPosition, Voxel>) -> Self {
        Self(grid)
    }
}

impl From<Grid> for HashMap<GridPosition, Voxel> {
    fn from(grid: Grid) -> Self {
        grid.0
    }
}

impl FromIterator<(GridPosition, Voxel)> for Grid {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (GridPosition, Voxel)>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for Grid {
    type Item = (GridPosition, Voxel);
    type IntoIter = hash_map::IntoIter<GridPosition, Voxel>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Grid {
    type Item = (&'a GridPosition, &'a Voxel);
    type IntoIter = hash_map::Iter<'a, GridPosition, Voxel>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Grid {
    type Item = (&'a GridPosition, &'a mut Voxel);
    type IntoIter = hash_map::IterMut<'a, GridPosition, Voxel>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[derive(Debug)]
pub enum VoxelAssetError {
    Utf8Error { source: std::str::Utf8Error },
    InvalidCsvLine { line: usize, source: CsvLineError },
}

impl VoxelAsset {
    pub fn from_bytes(bytes: &[u8]) -> Result<VoxelAsset, VoxelAssetError> {
        let data = str::from_utf8(bytes).map_err(|e| VoxelAssetError::Utf8Error { source: e })?;
        Self::parse_csv(data)
    }

    fn parse_csv(data: &str) -> Result<VoxelAsset, VoxelAssetError> {
        let mut grid = Grid::default();

        let mut colors: HashSet<PaletteColor> = HashSet::new();

        for (idx, line) in data.lines().enumerate().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            let CsvLine { x, y, z, r, g, b } =
                CsvLine::try_from(line).map_err(|e| VoxelAssetError::InvalidCsvLine {
                    line: idx + 1,
                    source: e,
                })?;

            let color = Color::Srgb(Srgb { r, g, b });

            let palette_color = PaletteColor::from(color);
            colors.insert(palette_color.clone());

            grid.insert(GridPosition { x, y, z }, Voxel { palette_color });
        }

        Ok(VoxelAsset {
            grid,
            colors: colors.into_iter().collect::<Vec<PaletteColor>>(),
        })
    }
}

struct CsvLine {
    x: i32,
    y: i32,
    z: i32,
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug)]
pub enum CsvLineError {
    MissingColumn(&'static str),
    InvalidColumn {
        field: &'static str,
        expected: &'static str,
        source: String,
    },
    UnknownColumn,
}

impl TryFrom<&str> for CsvLine {
    type Error = CsvLineError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut fields = value.split(',');

        let x = Self::parse_field(&mut fields, "x", "i32")?;
        let y = Self::parse_field(&mut fields, "y", "i32")?;
        let z = Self::parse_field(&mut fields, "z", "i32")?;
        let r = Self::parse_field(&mut fields, "r", "u8")?;
        let g = Self::parse_field(&mut fields, "g", "u8")?;
        let b = Self::parse_field(&mut fields, "b", "u8")?;

        if fields.next().is_some() {
            return Err(CsvLineError::UnknownColumn);
        }

        Ok(CsvLine { x, y, z, r, g, b })
    }
}

impl CsvLine {
    fn parse_field<T>(
        fields: &mut std::str::Split<'_, char>,
        field: &'static str,
        expected: &'static str,
    ) -> Result<T, CsvLineError>
    where
        T: std::str::FromStr,
        T::Err: ToString,
    {
        fields
            .next()
            .ok_or(CsvLineError::MissingColumn(field))?
            .trim()
            .parse::<T>()
            .map_err(|e| CsvLineError::InvalidColumn {
                field,
                expected,
                source: e.to_string(),
            })
    }
}
