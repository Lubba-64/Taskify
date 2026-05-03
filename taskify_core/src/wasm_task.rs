#[cfg(target_arch = "wasm32")]
use futures::Future;
#[cfg(target_arch = "wasm32")]
use futures::FutureExt;
#[cfg(target_arch = "wasm32")]
use std::cell::Cell;
#[cfg(target_arch = "wasm32")]
use std::panic;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::thread;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;

pub struct TaskHandler<T: Clone + 'static> {
    obj: Option<T>,
    #[cfg(target_arch = "wasm32")]
    wasm_task: Task<T>,
}

unsafe impl<T: Clone + 'static> std::marker::Sync for TaskHandler<T> {}

impl<T: Clone + 'static> TaskHandler<T> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(obj: T) -> TaskHandler<T> {
        Self { obj: Some(obj) }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(obj: impl Future<Output = T> + 'static) -> TaskHandler<T> {
        Self {
            obj: None::<T>,
            wasm_task: Task::new(obj),
        }
    }

    pub fn poll(&mut self) -> Option<T> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(output) = self.wasm_task.take_output() {
                self.obj = Some(output.expect("file dialog async task panicked"));
            }
            self.obj.clone()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.obj.clone()
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct Task<T>(Rc<Cell<Option<thread::Result<T>>>>);

#[cfg(target_arch = "wasm32")]
impl<T: 'static> Task<T> {
    pub fn new<F: 'static + Future<Output = T>>(future: F) -> Self {
        let sender = Rc::new(Cell::new(None));
        let receiver = sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let future = panic::AssertUnwindSafe(future).catch_unwind();
            sender.set(Some(future.await));
        });
        Self(receiver)
    }
    pub fn take_output(&self) -> Option<thread::Result<T>> {
        self.0.take()
    }
}
