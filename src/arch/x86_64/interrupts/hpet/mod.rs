#![allow(dead_code)]

mod hpet;
mod hpet_timer;
mod mmio;
mod registers;

use crate::*;
use conquer_once::spin::OnceCell;
use core::sync::atomic::{AtomicU64, Ordering};
use hpet::*;
use registers::*;
use x86_64::VirtAddr;

/// Try to initialize the HPET.
pub unsafe fn init(hpet_address: VirtAddr) {
    let hpet_mmio = hpet_address.as_mut_ptr::<HpetRegister>();
    let hpet_instance = Hpet::new(hpet_mmio);

    // First disable the hpet if it is running.
    if hpet_instance.is_enabled() {
        hpet_instance.disable();
        hpet_instance.set_main_counter_value(0);
    }

    // We don't need the HPET has it's useless for us.
    if !hpet_instance.has_legacy_mapping() {
        panic!("HPET doesn't have legacy mapping.")
    }

    let main_timer = hpet_instance.get_timer(0).expect("Main timer not found.");

    // Timer must suppoer periodic interrupt
    if !main_timer.support_periodic_interrupt() {
        panic!("HPET timer doesn't support periodic interrupt.")
    }

    let irq_period_ms: u64 = 1000 * 10;
    let irq_period_ns = irq_period_ms * 1_000_000;
    let irq_period_fs = irq_period_ns * 1_000_000;
    kernel_info!("HPET frequency: {} Hz", hpet_instance.get_frequency());
    kernel_info!("HPET IRQ period: {} fs", irq_period_fs);

    // IO-APIC expects edge triggering by default.
    main_timer.set_edge_trigger();
    // main_timer.set_periodic_mode();
    main_timer.enable_interrupt();
    // main_timer.set_accumulator_value(irq_period_tick);
    // main_timer.set_comparator_value(irq_period_tick);

    main_timer.set_interrupt_route(8);

    // Setup timer links
    MAIN_TIMER
        .try_init_once(|| Hpet::new(hpet_mmio))
        .expect("Timer can only be set once");
    HPET_PERIOD
        .try_init_once(|| Hpet::new(hpet_mmio).get_period() as u64)
        .expect("HPET Period can only be called once");
    crate::devices::timer::set_timer_provider(time_provider);

    crate::devices::timer::set_timer_registrar(timer_registrar);

    // Clear the interrupt state.
    hpet_instance.enable();
}

static MAIN_TIMER: OnceCell<Hpet> = OnceCell::uninit();
static HPET_PERIOD: OnceCell<u64> = OnceCell::uninit(); // period in fs (10 ^ -15)
static CURRENT_COMPARATOR: AtomicU64 = AtomicU64::new(u64::MAX);

unsafe impl Sync for Hpet {}
unsafe impl Send for Hpet {}

fn time_provider() -> u64 {
    let period = HPET_PERIOD.get().expect("HPET unitialized.");
    let counter = MAIN_TIMER
        .get()
        .expect("HPET unitialized")
        .get_main_counter_value();
    counter * period / 1_000_000
}

/// Sets the comparator only if > current counter and < current comparator - allowed_skew (ns)
fn timer_registrar(target_time: u64, allowed_skew: u64) {
    let period = HPET_PERIOD.get().expect("HPET unitialized.");
    let allowed_skew = allowed_skew * 1_000_000 / period;
    let timer = MAIN_TIMER.get().unwrap().get_timer(0).unwrap();
    let target_comparator = target_time * 1_000_000 / period;

    let current_main_counter = MAIN_TIMER
        .get()
        .expect("HPET unitialized")
        .get_main_counter_value();
    if current_main_counter + allowed_skew > target_comparator {
        // To prevent race conditions, just activate the timer
        crate::devices::timer::add_notification();
        return;
    }

    loop {
        let current_comparator = CURRENT_COMPARATOR.load(Ordering::SeqCst);
        if current_comparator > current_main_counter && current_comparator < target_comparator {
            // We will be woken earlier than needed. So, don't update it.
            return;
        }
        let org = CURRENT_COMPARATOR.compare_and_swap(
            current_comparator,
            target_comparator,
            Ordering::SeqCst,
        );
        if org == current_comparator {
            // Successful set
            timer.set_comparator_value(target_comparator);
            break;
        }
    }
}
