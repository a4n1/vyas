use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use vyas::prelude::*;

use crate::storage::Storage;

const LOCAL_STORAGE_KEY: &str = "history";
const CAPACITY: usize = 100;

#[derive(Debug, Clone)]
pub(crate) struct History {
    storage: Option<Storage>,
    state: State,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct State {
    undo_stack: VecDeque<Vec<Action>>,
    redo_stack: Vec<Vec<Action>>,
}

impl History {
    pub(crate) fn new() -> Self {
        let storage = Storage::new()
            .inspect_err(|err| {
                log::warn!("failed to initialize storage: {err}");
            })
            .ok();
        let state = State::new(storage.as_ref());

        Self { storage, state }
    }

    pub(crate) fn insert_actions(&mut self, actions: Vec<Action>) {
        self.state.undo_stack.push_back(actions);
        self.state.redo_stack.clear();
        self.save();
    }

    pub(crate) fn undo(&mut self) -> Option<Vec<Action>> {
        let actions = self.state.undo_stack.pop_back()?;
        self.state.redo_stack.push(actions.clone());
        self.save();
        Some(actions)
    }

    pub(crate) fn redo(&mut self) -> Option<Vec<Action>> {
        let actions = self.state.redo_stack.pop()?;
        self.state.undo_stack.push_back(actions.clone());
        self.save();
        Some(actions)
    }

    fn save(&mut self) {
        if self.state.undo_stack.len() > CAPACITY {
            self.state.undo_stack.pop_front();
        }

        if let Some(storage) = &self.storage {
            if let Err(err) = storage.save(LOCAL_STORAGE_KEY, &self.state) {
                log::warn!("failed to save {LOCAL_STORAGE_KEY} to local storage: {err}");
            }
        }
    }
}

impl State {
    fn new(storage: Option<&Storage>) -> Self {
        let Some(storage) = storage else {
            return Self::default();
        };

        match storage.load::<State>(LOCAL_STORAGE_KEY) {
            Ok(Some(state)) => state,
            Ok(None) => Self::default(),
            Err(err) => {
                log::warn!(
                    "failed to seed {LOCAL_STORAGE_KEY} state from from local storage: {err}"
                );
                Self::default()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Action {
    InsertVoxel {
        position: GridPosition,
        voxel: Voxel,
    },
    DeleteVoxel {
        position: GridPosition,
        voxel: Voxel,
    },
}
