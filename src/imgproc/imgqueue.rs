use camera::MonoCameraFrame;
use camera::MonoPixel;

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;

/// A simple queue with a way to add images when they are received,
/// and pull them off the queue and process them.
///

#[derive(Clone)]
pub struct ImageQueue<T>
where
    T: MonoPixel + 'static,
{
    framelist: Arc<Mutex<Vec<MonoCameraFrame<T>>>>,
    framelistsync: Arc<(Mutex<()>, Condvar)>,
}

impl<T> ImageQueue<T>
where
    T: MonoPixel + 'static,
{
    pub fn new() -> Self {
        ImageQueue {
            framelist: Arc::new(Mutex::new(Vec::new())),
            framelistsync: Arc::new((Mutex::new(()), Condvar::new())),
        }
    }

    /// Start the image processing chain when a frame is available
    pub fn add_frame_to_queue(&self, frame: MonoCameraFrame<T>) {
        let (lock, cvar) = &*self.framelistsync;
        let _guard = lock.lock().unwrap();
        self.framelist.lock().unwrap().push(frame);
        cvar.notify_one();
    }

    pub fn start<F>(&self, procfunc: F)
    where
        F: Fn(MonoCameraFrame<T>) + Send + 'static,
    {
        let framelistsync = self.framelistsync.clone();
        let framelist = self.framelist.clone();
        let _thread = thread::spawn(move || {
            loop {
                let (lock, cvar) = &*framelistsync;
                let guard = lock.lock().unwrap();
                let _unused = cvar.wait(guard).unwrap();
                while let Some(frame) = framelist.lock().unwrap().pop() {
                    // Process the frame
                    procfunc(frame);
                }
            }
        });
    }
}
