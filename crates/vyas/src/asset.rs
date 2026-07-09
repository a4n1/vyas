use std::collections::HashMap;

use crate::{
    color::{Color, Srgb},
    position::GridPosition,
    voxel::Voxel,
};

pub struct VoxelAsset {
    pub grid: Grid,
}

pub type Grid = HashMap<GridPosition, Voxel>;

#[derive(Debug)]
pub enum VoxelAssetError {
    Utf8Error { source: std::str::Utf8Error },
    InvalidCsvLine { line: usize, source: CsvLineError },
}

impl VoxelAsset {
    pub fn from_bytes(bytes: &[u8]) -> Result<VoxelAsset, VoxelAssetError> {
        let data = str::from_utf8(bytes).map_err(|e| VoxelAssetError::Utf8Error { source: e })?;
        let grid = Self::parse_csv(data)?;
        Ok(VoxelAsset { grid })
    }

    fn parse_csv(data: &str) -> Result<Grid, VoxelAssetError> {
        let mut grid = HashMap::new();

        for (idx, line) in data.lines().enumerate().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            let CsvLine { x, y, z, r, g, b } =
                CsvLine::try_from(line).map_err(|e| VoxelAssetError::InvalidCsvLine {
                    line: idx + 1,
                    source: e,
                })?;

            grid.insert(
                GridPosition { x, y, z },
                Voxel {
                    color: Color::Srgb(Srgb { r, g, b }),
                },
            );
        }

        Ok(grid)
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
