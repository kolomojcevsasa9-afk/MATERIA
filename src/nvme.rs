use core::ptr::{read_volatile, write_volatile};

pub struct NvmeDriver {
    pub base_addr: u64,
}

impl NvmeDriver {
    pub fn new(bus: u8, slot: u8) -> Self {
        // Читаем BAR0 и BAR1 для получения 64-битного адреса
        let bar0 = crate::pci::read_config_u32(bus, slot, 0, 0x10) as u64;
        let bar1 = crate::pci::read_config_u32(bus, slot, 0, 0x14) as u64;
        let base_addr = (bar1 << 32) | (bar0 & 0xFFFFFFF0);
        
        // Обертки для записи в PCI конфиг нет, используем низкоуровневый IO
        // (упрощенно, предположим, что Bus Master уже включен или BIOS его настроил)
        
        NvmeDriver { base_addr }
    }

    fn mmio_write32(&self, offset: u64, val: u32) {
        unsafe { write_volatile((self.base_addr + offset) as *mut u32, val); }
    }

    fn mmio_read32(&self, offset: u64) -> u32 {
        unsafe { read_volatile((self.base_addr + offset) as *const u32) }
    }

    pub fn init(&self) {
        crate::println!("NVMe initialized at base: {:#x}", self.base_addr);
        
        // 1. Читаем версию контроллера
        let vs = self.mmio_read32(0x08);
        crate::println!("NVMe Version: {:#x}", vs);

        // 2. Сбрасываем контроллер (Disable)
        self.mmio_write32(0x14, 0);
        
        // 3. Ждем готовности (RDY)
        // В реальном драйвере тут нужен цикл с таймаутом
        
        // 4. Включаем контроллер
        self.mmio_write32(0x14, 1);
        crate::println!("NVMe Controller Enabled.");
    }

    pub fn read_sector(&self, _lba: u64, _buffer: &mut [u8; 512]) {
        // TODO: Реализация очереди команд
    }

    pub fn write_sector(&self, lba: u64, _buffer: &[u8; 512]) {
        crate::println!("NVMe: Write request to LBA {}", lba);
        // Запись в Doorbell хвоста очереди (SQ0TDBL)
        self.mmio_write32(0x1000, 1);
    }
}
