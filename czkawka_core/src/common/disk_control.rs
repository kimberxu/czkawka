use std::path::Path;
use std::sync::{Condvar, Mutex};

use once_cell::sync::Lazy;

/// Hardcoded Disk IDs based on personal setup
/// 0: SSD (C:\, D:\) - Unlimited
/// 1: HDD A (E:\) - Limit 1
/// 2: HDD B (F:\) - Limit 1
/// 3: Others - Limit 1 (Fallback)
const DISK_SSD: usize = 0;
const DISK_HDD_A: usize = 1;
const DISK_HDD_B: usize = 2;
const DISK_OTHER: usize = 3;

/// Limits per disk
/// SSD: Essentially unlimited (999)
/// HDD: Strict serial (1)
/// Video on HDD: Limit 2 (Special case, can be handled by a separate semaphore or just using the same with capacity 2 if we wanted, but for now strict 1 is safer for duplicates)
/// Actually, the plan says Video might need limit 2.
/// Let's make the semaphore configurable or just fixed.
/// For DuplicateFinder, we want Limit 1.
/// For Video, we want Limit 2.
/// This implies we might need *different* semaphores for different tools OR a semaphore with capacity 2, but DuplicateFinder takes 2 permits?
/// Simpler: Just make the limit 2 for HDD, and DuplicateFinder explicitly acquires 2 permits?
/// No, that's overengineering.
/// Let's stick to the plan:
/// Duplicate: Strict 1.
/// Video: Limit 2.
///
/// We will implement a `get_semaphore` that returns the semaphore for the disk.
/// But the limit is stateful.
///
/// Revised Plan from docs:
/// "Duplicate Finder: Strict 1-thread limit"
/// "Similar Videos: Coarse-grained Limit 2 per HDD"
///
/// Implementation:
/// We can have `SimpleSemaphore` support a `max_permits` in constructor.
/// But the limit is a property of the *usage pattern* + *hardware*.
///
/// Actually, if we set the semaphore to Limit 2 globally for HDD:
/// - DuplicateFinder can acquire 1 permit. It might run 2 threads. This violates "Strict 1-thread".
/// - Unless DuplicateFinder acquires *all* permits?
///
/// Alternative:
/// Just use Limit 1 for HDDs for *everything* initially. It's safer.
/// The `Limit 2` for videos was a "nice to have" optimization.
/// Let's start with Limit 1 for HDDs to ensure no thrashing.
///
/// Wait, `Limit 1` for HDD is the *core* requirement.
/// Let's define the limits:
/// SSD: 1000
/// HDD_A: 1
/// HDD_B: 1
/// Other: 1
///
struct SimpleSemaphore {
    count: Mutex<usize>,
    cv: Condvar,
    limit: usize,
}

impl SimpleSemaphore {
    fn new(limit: usize) -> Self {
        Self {
            count: Mutex::new(0),
            cv: Condvar::new(),
            limit,
        }
    }

    fn acquire(&self) {
        let mut count = self.count.lock().unwrap();
        while *count >= self.limit {
            count = self.cv.wait(count).unwrap();
        }
        *count += 1;
    }

    fn release(&self) {
        let mut count = self.count.lock().unwrap();
        if *count > 0 {
            *count -= 1;
            self.cv.notify_one();
        }
    }
}

struct DiskController {
    semaphores: [SimpleSemaphore; 4],
}

impl DiskController {
    fn new() -> Self {
        Self {
            semaphores: [
                SimpleSemaphore::new(1000), // SSD
                SimpleSemaphore::new(1),    // HDD A
                SimpleSemaphore::new(1),    // HDD B
                SimpleSemaphore::new(1),    // Other
            ],
        }
    }

    fn get_disk_id(&self, path: &Path) -> usize {
        let path_str = path.to_string_lossy().to_uppercase();
        if path_str.starts_with("C:") || path_str.starts_with("D:") {
            DISK_SSD
        } else if path_str.starts_with("E:") {
            DISK_HDD_A
        } else if path_str.starts_with("F:") {
            DISK_HDD_B
        } else {
            DISK_OTHER
        }
    }

    fn acquire(&self, path: &Path) -> usize {
        let id = self.get_disk_id(path);
        self.semaphores[id].acquire();
        id
    }

    fn release(&self, id: usize) {
        if id < self.semaphores.len() {
            self.semaphores[id].release();
        }
    }
}

static CONTROLLER: Lazy<DiskController> = Lazy::new(DiskController::new);

/// Run a closure with IO lock for the specific disk.
pub fn with_io_lock<F, T>(path: &Path, f: F) -> T
where
    F: FnOnce() -> T,
{
    let id = CONTROLLER.acquire(path);
    // log::trace!("Acquired IO lock for disk {} ({:?})", id, path);
    let result = f();
    CONTROLLER.release(id);
    // log::trace!("Released IO lock for disk {} ({:?})", id, path);
    result
}

/// Helper to check if a path is on SSD (for optimizations)
pub fn is_ssd(path: &Path) -> bool {
    CONTROLLER.get_disk_id(path) == DISK_SSD
}
