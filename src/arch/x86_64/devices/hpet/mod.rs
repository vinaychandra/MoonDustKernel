//! High Performance Event Timer (HPET) driver support.

#![allow(dead_code)]

mod hpet;
mod hpet_timer;
mod mmio;
mod registers;

use chrono::Duration;
use conquer_once::spin::OnceCell;
use hpet::*;
use registers::*;
use x86_64::VirtAddr;

static HPET_INSTANCE: OnceCell<Hpet> = OnceCell::uninit();

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
    // TODO: Should this be removed?
    if !hpet_instance.has_legacy_mapping() {
        panic!("HPET doesn't have legacy mapping.")
    }

    let main_timer = hpet_instance.get_timer(0).expect("Main timer not found.");

    // Timer must suppoer periodic interrupt
    if !main_timer.support_periodic_interrupt() {
        panic!("HPET timer doesn't support periodic interrupt.")
    }

    let irq_period_ms = 1000;
    let irq_period_ns = irq_period_ms * 1_000_000;
    let irq_period_fs = irq_period_ns * 1_000_000;
    info!(
        target: "hpet",
        "HPET frequency: {} Hz and Period: {}",
        hpet_instance.get_frequency(),
        hpet_instance.get_period()
    );
    info!(target: "hpet", "HPET IRQ period: {} ms", irq_period_ms);

    let irq_period_tick = irq_period_fs / u64::from(hpet_instance.get_period());

    // IO-APIC expects edge triggering by default.
    main_timer.set_edge_trigger();
    main_timer.set_periodic_mode();
    main_timer.enable_interrupt();
    main_timer.set_accumulator_value(irq_period_tick);
    main_timer.set_comparator_value(irq_period_tick);

    main_timer.set_interrupt_route(8);
    // main_timer.set_interrupt_route(2);

    // Clear the interrupt state.
    hpet_instance.enable();

    // Store the instance
    HPET_INSTANCE.init_once(|| hpet_instance);
    crate::common::time::UPTIME_PROVIDER.init_once(|| time_from_startup);
}

// TODO: Support overflow.
fn time_from_startup() -> Duration {
    if let Some(hpet) = HPET_INSTANCE.get() {
        let tick_in_nanos = hpet.get_period() / 1000_000;
        let raw = hpet.get_main_counter_value();
        Duration::nanoseconds(raw as i64 * tick_in_nanos as i64)
    } else {
        Duration::zero()
    }
}
