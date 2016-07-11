use memory::PAGE_SIZE;
use memory::Frame;
use memory::FrameAllocator;
use memory::paging::table::*;
pub const ENTRY_COUNT: usize = 512;
use core::ptr::Unique;
pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub mod table;

pub struct Page {
    number: usize,
}

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}


pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
            self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(PRESENT) {
            Some(Frame::containing_address (
                    self.0 as usize & 0x000fffff_fffff000
                ))
        }
        else {
            None
        }
    }
    
    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        assert!(frame.start_address() & !0x000fffff_fffff000 == 0); // _ is a visual separator for number literals
        self.0 = (frame.start_address() as u64) | flags.bits();
    }

}

bitflags! {
    pub flags EntryFlags: u64 {
        const PRESENT =         1 << 0,
        const WRITABLE =        1 << 1,
    const USER_ACCESSIBLE = 1 << 2,
    const WRITE_THROUGH =   1 << 3,
    const NO_CACHE  =       1 << 4,
    const ACCESSED  =       1 << 5,
    const DIRTY     =       1 << 6,
    const HUGE_PAGE =       1 << 7,
    const GLOBAL    =       1 << 8,
    const NO_EXECUTE =      1 << 63,
}
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_000, "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }
}



impl Page {

fn start_address(&self) -> usize {
    self.number * PAGE_SIZE
}

fn p4_index(&self) -> usize {
    (self.number >> 27) & 0o777
}

fn p3_index(&self) -> usize {
    (self.number >> 18) & 0o777
}

fn p2_index(&self) -> usize {
    (self.number >> 9) & 0o777
}

fn p1_index(&self) -> usize {
    (self.number >> 0) & 0o777
}

}

impl ActivePageTable {
pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new(table::P4),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.get() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.get_mut() }
    }
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress>
    {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address)).map(|frame| frame.number * PAGE_SIZE + offset)
    }
    
    fn translate_page(&self, page: Page) -> Option<Frame>
    {
    
    
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                 let p3_entry = &p3[page.p3_index()];
                if let Some(start_frame) = p3_entry.pointed_frame() {
                     if p3_entry.flags().contains(HUGE_PAGE) {
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some (Frame {
                            number: start_frame.number + page.p2_index() * ENTRY_COUNT +
                                page.p1_index(),
                            })
                    }
                }
            
                if let Some(p2) = p3.next_table(page.p3_index()) {
                     let p2_entry = &p2[page.p2_index()];
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                         assert!(start_frame.number % ENTRY_COUNT == 0);
                        return Some (Frame {
                             number: start_frame.number + page.p1_index()
                        });
                    }
                }
             None
            })

    };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(huge_page)
}
 
    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags,
                     allocator: &mut A)
        where A: FrameAllocator
    {
        let p4 = self.p4_mut();
        let mut p3 = p4.next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
            where A: FrameAllocator
    {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
            where A: FrameAllocator
    {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
                    .next_table_mut(page.p4_index())
                    .and_then(|p3| p3.next_table_mut(page.p3_index()))
                    .and_then(|p2| p2.next_table_mut(page.p2_index()))
                    .expect("Mapping code does not support huge pages");
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        allocator.deallocate_frame(frame)
    }

}


pub fn test_paging<A>(allocator: &mut A)
        where A: FrameAllocator
{
    let page_table = unsafe { ActivePageTable::new() };
    // address 0 is mapped
    println!("Some = {:?}", page_table.translate(0));
    //  // second P1 entry
    println!("Some = {:?}", page_table.translate(4096));
    //  // second P2 entry
    println!("Some = {:?}", page_table.translate(512 * 4096));
    //  // 300th P2 entry
    println!("Some = {:?}", page_table.translate(300 * 512 * 4096));
    //  // second P3 entry
    println!("None = {:?}", page_table.translate(512 * 512 * 4096));
    //  // last mapped byte
    println!("Some = {:?}", page_table.translate(512 * 512 * 4096 - 1));
    
}
