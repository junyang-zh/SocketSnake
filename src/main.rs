pub mod yard;
pub mod render;

use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        let mut y = yard::YardSim::new(30, 20, 3, 3);
        let mut ui = render::TUIHelper::new();
        y.cleanup();
        y.init_snake();
        y.init_snake();
        for _i in 0..20 {
            thread::sleep(Duration::from_millis(100));
            y.next_tick();
            y.cleanup();
            ui.refresh_yard(y.generate_buf()).unwrap();
        }
    });
    handle.join().unwrap();
}
