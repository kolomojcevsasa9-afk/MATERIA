use crate::nvme::NvmeDriver;
use spin::Mutex;

const BLOCK_SIZE: usize = 512;

pub struct SimpleFileSystem {
    driver: Option<NvmeDriver>,
    initialized: bool,
}

static SFS: Mutex<SimpleFileSystem> = Mutex::new(SimpleFileSystem {
    driver: None,
    initialized: false,
});

impl SimpleFileSystem {
    pub fn set_driver(&mut self, driver: NvmeDriver) {
        self.driver = Some(driver);
    }

    fn read_block(&self, lba: u64, buffer: &mut [u8; 512]) {
        if let Some(ref d) = self.driver {
            d.read_sector(lba, buffer);
        }
    }

    fn write_block(&self, lba: u64, buffer: &[u8; 512]) {
        if let Some(ref d) = self.driver {
            d.write_sector(lba, buffer);
        }
    }

    pub fn init(&mut self) {
        if self.initialized { return; }
        self.initialized = true;
        crate::println!("SFS initialized with NVMe driver.");
    }
}

pub fn init(driver: NvmeDriver) {
    let mut sfs = SFS.lock();
    sfs.set_driver(driver);
    sfs.init();
}

pub fn touch(_filename: &str) -> bool { true }
pub fn mkdir(_dirname: &str) -> bool { true }
pub fn ls() { crate::println!("SFS: ls (unimplemented with NVMe)"); }
pub fn cat(_filename: &str) { crate::println!("SFS: cat (unimplemented with NVMe)"); }
pub fn rm(_filename: &str) -> bool { true }
pub fn write_file(_filename: &str, _content: &str) -> bool { true }
