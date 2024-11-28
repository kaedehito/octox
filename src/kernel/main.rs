#![no_std]
#![no_main]

extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering};
use kernel::{
    bio, console, end, hart, kalloc, kmain, msg, null, plic, println,
    proc::{self, scheduler, user_init, Cpus},
    trap, virtio_disk, vm,
};

static STARTED: AtomicBool = AtomicBool::new(false);

kmain!(main);

extern "C" fn main() -> ! {
    let cpuid = unsafe { Cpus::cpu_id() };
    if cpuid == 0 {
        let initcode = include_bytes!(concat!(env!("OUT_DIR"), "/bin/_initcode"));
        console::init(); // console init
        println!("");
        println!("octox kernel is booting");
        println!("");

        null::init(); // null device init
        end!("null::init");

        kalloc::init(); // physical memory allocator
        end!("kalloc::init");

        vm::kinit(); // create kernel page table
        end!("vm::init");

        vm::kinithart(); // turn on paging
        end!("vm::kinithart");

        proc::init(); // process table
        end!("proc::init");

        trap::inithart(); // install kernel trap vector
        end!("trap::inithart");

        plic::init(); // set up interrupt controller
        end!("plic::init");

        plic::inithart(); // ask PLIC for device interrupts
        end!("plic::inithart");

        bio::init(); // buffer cache
        end!("bio::init");

        virtio_disk::init(); // emulated hard disk
        end!("virtio_disk::init");

        user_init(initcode);
        end!("user_init");

        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {
            core::hint::spin_loop()
        }
        hart!("hart {} starting", unsafe { Cpus::cpu_id() });

        vm::kinithart(); // turn on paging
        end!("turn on paging");

        trap::inithart(); // install kernel trap vector
        end!("install kernel trap vector");

        plic::inithart(); // ask PLIC for device interrupts
        end!("ask PLIC for device interrupts");

        hart!("hart {}: ok", unsafe { Cpus::cpu_id() });
    }
    scheduler()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    println!("\x1b[31;1m===[\tPANIC\t]===\x1b[0m");
    use kernel::printf::panic_inner;
    panic_inner(info)
}
