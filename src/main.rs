fn main() {
    let program = "+ + * - /";
    let machine_code = jit(program);
    let result = unsafe { run(&machine_code) };
    println!(
        "The program \"{}\" calculates the value {}",
        program, result
    );
}

// Parses a calculator "program" and returns a sequence of bytes
// corresponding to the equivalent machine code
fn jit(program: &str) -> Vec<u8> {
    todo!()
}

// Runs the machine code (provided as a byte slice) and returns the resulting value
//
/// # Safety
///
/// `machine_code` must be valid machine code in the C ABI that returns an `i64`.
unsafe fn run(machine_code: &[u8]) -> i64 {
    fn perror(func: &str) -> ! {
        panic!("error in {func}: {}", std::io::Error::last_os_error());
    }

    use core::{ffi::c_void, mem, ptr, slice};
    const PAGE_SIZE: usize = 4096;
    let mmap_len = machine_code.len().div_ceil(PAGE_SIZE) * PAGE_SIZE;

    // SAFETY: We're making an anonymous mapping here (of length mmap_len)
    // See mmap(2) for details
    let mmap_ptr = unsafe {
        libc::mmap(
            ptr::null_mut(),
            mmap_len,
            libc::PROT_WRITE | libc::PROT_READ,
            libc::MAP_ANONYMOUS,
            0,
            0,
        )
    };
    if mmap_ptr == libc::MAP_FAILED {
        perror("mmap");
    }
    {
        // SAFETY: mmap_ptr is a valid ptr of length mmap_len and is valid until the end of this block
        let map_slice = unsafe { slice::from_raw_parts_mut(mmap_ptr.cast::<u8>(), mmap_len) };
        map_slice[0..machine_code.len()].copy_from_slice(machine_code);
    }

    // Make the memory readonly but executable for security (W^X) https://en.wikipedia.org/wiki/W%5EX
    // SAFETY: mmap_ptr is a valid ptr of length mmap_len
    // See mprotect(2) for details
    let mprotect_res =
        unsafe { libc::mprotect(mmap_ptr, mmap_len, libc::PROT_EXEC | libc::PROT_READ) };
    if mprotect_res < 0 {
        perror("mprotect");
    }

    // Cast to a C function pointer and call it
    let result = {
        // SAFETY: now having written machine_code to mmap_ptr and made it executable, mmap_ptr is now a valid C function.
        let func = unsafe { mem::transmute::<*mut c_void, extern "C" fn() -> i64>(mmap_ptr) };

        func()
    };

    // Free the memory!
    // SAFETY: mmap_ptr is a valid ptr of length mmap_len
    let munmap_res = unsafe { libc::munmap(mmap_ptr, mmap_len) };
    if munmap_res < 0 {
        perror("munmap");
    }

    result
}
