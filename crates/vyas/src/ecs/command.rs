use std::cell::RefCell;

use super::{Bundle, Entity, World};

pub type CommandQueue = RefCell<Vec<Box<dyn FnOnce(&mut World)>>>;

pub struct Commands {
    pub(crate) queue: *const CommandQueue,
}

impl Commands {
    pub fn spawn<B: Bundle>(&mut self, bundle: B) {
        let queue = unsafe { &*self.queue };
        queue.borrow_mut().push(Box::new(move |world| {
            let _ = world.spawn(bundle);
        }));
    }

    pub fn despawn(&mut self, entity: Entity) {
        let queue = unsafe { &*self.queue };
        queue
            .borrow_mut()
            .push(Box::new(move |world| world.despawn(entity)));
    }
}
