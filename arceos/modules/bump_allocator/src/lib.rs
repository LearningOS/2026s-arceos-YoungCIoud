#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    // set ture when init successfully
    valid: bool,

    // [start, end)
    start: usize,
    end: usize,

    bytes_pos: usize,

    pages_pos: usize,

    count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            valid: false,
            start: 0,
            end: 0,
            bytes_pos: 0,
            pages_pos: 0,
            count: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.bytes_pos = start;
        self.pages_pos = start + size;
        self.count = 0;
        self.valid = true;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> allocator::AllocResult {
        Err(allocator::AllocError::NoMemory) // unsupported
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let (len, align) = (layout.size(), layout.align());
        let len = align_up(len, align);
        self.bytes_pos = align_up(self.bytes_pos, align);


        if self.bytes_pos + len > self.pages_pos {
            return Err(allocator::AllocError::NoMemory);
        }

        let ptr = core::ptr::NonNull::new(self.bytes_pos as *mut u8);
        self.bytes_pos += len;
        self.count += 1;

        ptr.ok_or(allocator::AllocError::NoMemory)
    }

    fn dealloc(&mut self, _pos: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        self.count -= 1;
        if self.count == 0 {
            // 回收所有空间
            self.bytes_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.bytes_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.pages_pos - self.bytes_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        if num_pages == 0 ||align_pow2 == 0 || !align_pow2.is_power_of_two() || align_pow2 % SIZE != 0 {
            return Err(allocator::AllocError::InvalidParam);
        }
        let len = num_pages
            .checked_mul(SIZE)
            .ok_or(allocator::AllocError::NoMemory)?;

        self.pages_pos = align_down(self.pages_pos, align_pow2);
        if self.bytes_pos + len > self.pages_pos {
            return Err(allocator::AllocError::NoMemory);
        }

        self.pages_pos = self.pages_pos
            .checked_sub(len)
            .ok_or(allocator::AllocError::NoMemory)?;

        Ok(self.pages_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        panic!("dealloc_pages is unsupported")
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.pages_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        (self.pages_pos - self.start) / SIZE
    }
}

#[inline]
const fn align_down(pos: usize, align: usize) -> usize {
    pos & !(align - 1)
}

#[inline]
const fn align_up(pos: usize, align: usize) -> usize {
    (pos + align - 1) & !(align - 1)
}