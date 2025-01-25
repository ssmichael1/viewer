use crate::cameraframe::CameraFrame;
use crate::cameraframe::FrameData;
use std::thread;

pub struct SimSource {
    thread: Option<thread::JoinHandle<()>>,
}

fn test_data(xoffset: f64, yoffset: f64) -> FrameData<u16> {
    use rand::distributions::Distribution;
    use rand_distr::Normal;

    let normal = Normal::new(0.0, 200.0).unwrap();
    let mut rng = rand::thread_rng();

    let width = 1024;
    let height = 768;
    let npixels = width * height;
    FrameData::<u16> {
        width,
        height,
        data: (0..npixels)
            .map(|x| {
                let mut row = (x % width) as f64;
                let mut col = (x / width) as f64;
                row -= 256.0;
                row -= yoffset;
                col -= 256.0;
                col -= xoffset;
                let g = f64::exp(-(row * row + col * col) / 400.0) * 2048.0;
                (g + 1024.0 + normal.sample(&mut rng)).clamp(0.0, 4095.0) as u16
            })
            .collect(),
    }
}

impl SimSource {
    pub fn new() -> Self {
        SimSource { thread: None }
    }

    pub fn start<F>(&mut self, onframe: F)
    where
        F: Fn(CameraFrame<u16>) + Send + 'static,
    {
        // Spawn a thread that continuously generates frames
        self.thread = Some(thread::spawn(move || {
            use std::f64::consts::PI;
            loop {
                thread::sleep(std::time::Duration::from_millis(30));
                let now = chrono::Utc::now();

                // Create a frame
                let xoffset = (now.timestamp_millis() as f64 * 2.0 * PI / 1000.0).cos() * 10.0;
                let yoffset = (now.timestamp_millis() as f64 * 2.0 * PI / 3000.0).cos() * 20.0;

                let frame = CameraFrame::<u16>::create(0.1, now, 12, test_data(xoffset, yoffset));
                // Run the callback
                onframe(frame);
            }
        }));
    }
}
