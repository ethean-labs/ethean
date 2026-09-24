//! Anonymous mappings via `mmap` (Unix) or `VirtualAlloc` (Windows).

use std::ptr;

/// `madvise` advice: disable transparent huge pages for the region. Consulted only on Linux
/// (a no-op elsewhere); see [`madvise`].
pub const MADV_NOHUGEPAGE: usize = 15;

/// Reserve `size` bytes of anonymous virtual address space, lazily backed by physical pages.
///
/// # Safety
/// Always safe to call; returns a page-aligned pointer or null on failure, and the caller owns the
/// resulting mapping.
#[inline]
pub unsafe fn mmap_anonymous(size: usize) -> *mut u8 {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let flags = libc::MAP_PRIVATE | libc::MAP_ANON;
        #[cfg(target_os = "linux")]
        let flags = flags | libc::MAP_NORESERVE;
        let ret = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                flags,
                -1,
                0,
            )
        };
        if ret == libc::MAP_FAILED {
            ptr::null_mut()
        } else {
            ret.cast::<u8>()
        }
    }
    #[cfg(windows)]
    {
        // MEM_RESERVE only: sparse reservation (leanVM arenas are huge). Pages are committed
        // on first touch via a later VirtualAlloc MEM_COMMIT over the used range is not done
        // here; for Windows we keep the arena disabled (see enable_arena) so this is unused.
        const MEM_RESERVE: u32 = 0x2000;
        const PAGE_READWRITE: u32 = 0x04;
        const PAGE_NOACCESS: u32 = 0x01;
        extern "system" {
            fn VirtualAlloc(
                lp_address: *mut u8,
                dw_size: usize,
                fl_allocation_type: u32,
                fl_protect: u32,
            ) -> *mut u8;
        }
        let _ = PAGE_READWRITE;
        let ret = unsafe { VirtualAlloc(ptr::null_mut(), size, MEM_RESERVE, PAGE_NOACCESS) };
        if ret.is_null() {
            ptr::null_mut()
        } else {
            ret
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        let _ = size;
        ptr::null_mut()
    }
}

/// Apply `advice` to `[ptr, ptr + size)`. No-op on non-Linux (the advice values we use are
/// Linux-specific).
///
/// # Safety
/// `ptr`/`size` must describe a live mapping returned by [`mmap_anonymous`].
#[inline]
pub unsafe fn madvise(ptr: *mut u8, size: usize, advice: usize) {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::madvise(ptr.cast::<libc::c_void>(), size, advice as libc::c_int);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (ptr, size, advice);
    }
}
