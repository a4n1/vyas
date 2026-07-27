use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use gloo_net::http::Request;
use vyas::prelude::*;

use crate::{browser::Browser, storage::Storage};

type AssetGrid = vyas::asset::Grid;

pub(crate) const GRID_SIZE: i32 = 64;

const LOCAL_STORAGE_KEY: &str = "last_save";

#[derive(Clone)]
pub(crate) struct Grid {
    state: State,
    storage: Option<Storage>,
}

type State = Rc<RefCell<AssetGrid>>;

impl Grid {
    pub(crate) fn new() -> Self {
        let storage = Storage::new()
            .inspect_err(|err| {
                log::warn!("failed to initialize storage: {err}");
            })
            .ok();

        let state = Rc::new(RefCell::new(AssetGrid::new()));

        Self { storage, state }
    }

    pub(crate) async fn init(&self) {
        if let Some(storage) = &self.storage {
            match storage.load::<AssetGrid>(LOCAL_STORAGE_KEY) {
                Ok(Some(grid)) => {
                    *self.state.borrow_mut() = grid;
                }
                Ok(None) => {
                    if let Some(example) = self.load_example().await {
                        *self.state.borrow_mut() = example;
                    }
                }
                Err(err) => {
                    log::warn!(
                        "failed to seed {LOCAL_STORAGE_KEY} state from local storage: {err}"
                    );
                }
            }
        }
    }

    pub async fn load_example(&self) -> Option<AssetGrid> {
        let response = match Request::get("/models/tree.csv").send().await {
            Ok(response) => response,
            Err(err) => {
                log::warn!("failed to fetch example model: {err}");
                return None;
            }
        };

        let text = match response.text().await {
            Ok(response) => response,
            Err(err) => {
                log::warn!("failed to get example model text: {err}");
                return None;
            }
        };

        let grid = match VoxelAsset::from_bytes(text.as_bytes()) {
            Ok(asset) => asset.grid,
            Err(e) => {
                log::warn!("failed to parse example asset: {e:#?}");
                return None;
            }
        };

        Some(denormalize_grid(&grid))
    }

    pub(crate) fn borrow(&self) -> Ref<'_, AssetGrid> {
        self.state.borrow()
    }

    pub(crate) fn insert(&self, position: GridPosition, voxel: Voxel) {
        self.state.borrow_mut().insert(position, voxel);
        Browser::trigger_grid_update();
        self.save();
    }

    pub(crate) fn remove(&self, position: &GridPosition) -> Option<Voxel> {
        let voxel = self.state.borrow_mut().remove(position);

        Browser::trigger_grid_update();
        self.save();

        voxel
    }

    pub(crate) fn set(&self, grid: AssetGrid) {
        *self.state.borrow_mut() = grid;
        Browser::trigger_grid_update();
        self.save();
    }

    fn save(&self) {
        if let Some(storage) = &self.storage {
            if let Err(err) = storage.save(LOCAL_STORAGE_KEY, &*self.state.borrow()) {
                log::warn!("failed to save {LOCAL_STORAGE_KEY} to local storage: {err}");
            }
        }
    }

    pub(crate) fn into_iter(&self) -> impl IntoIterator<Item = (GridPosition, Voxel)> {
        self.state.borrow().clone().into_iter()
    }

    pub(crate) fn normalize(&self) -> AssetGrid {
        normalize_grid(&self.state.borrow())
    }
}

pub(crate) fn grid_in_bounds(grid: &AssetGrid) -> bool {
    grid.keys().all(|position| {
        (0..GRID_SIZE).contains(&position.x)
            && (0..GRID_SIZE).contains(&position.y)
            && (0..GRID_SIZE).contains(&position.z)
    })
}

pub(crate) fn normalize_grid(grid: &AssetGrid) -> AssetGrid {
    grid.iter()
        .map(|(position, voxel)| {
            let position = GridPosition {
                x: position.x + GRID_SIZE / 2,
                y: position.y,
                z: position.z + GRID_SIZE / 2,
            };
            (position, voxel.clone())
        })
        .collect()
}

pub(crate) fn denormalize_grid(grid: &AssetGrid) -> AssetGrid {
    grid.iter()
        .map(|(position, voxel)| {
            let position = GridPosition {
                x: position.x - GRID_SIZE / 2,
                y: position.y,
                z: position.z - GRID_SIZE / 2,
            };
            (position, voxel.clone())
        })
        .collect()
}
