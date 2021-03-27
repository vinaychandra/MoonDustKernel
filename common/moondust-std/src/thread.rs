use alloc::{boxed::Box, sync::Arc};
use core::cell::UnsafeCell;
use moondust_sys::syscall::{ProcessControl, Syscalls};

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let my_result: Arc<UnsafeCell<Option<Result<T, ()>>>> = Arc::new(UnsafeCell::new(None));
    let target_result = my_result.clone();
    let f_closure = move || {
        let return_val = Ok(f());
        unsafe { *target_result.get() = Some(return_val) };
    };

    let boxed_val: Box<dyn FnOnce()> = Box::new(f_closure);
    let boxed_box = Box::new(boxed_val);
    let boxed_raw = Box::into_raw(boxed_box);
    extern "C" fn thread_start(main: *mut ()) -> () {
        unsafe {
            let val = Box::from_raw(main as *mut Box<dyn FnOnce()>);
            val();
        }

        let exit_call = Syscalls::Exit(0);
        exit_call.invoke();
    }

    let thread_syscall = Syscalls::Process(ProcessControl::CreateThread {
        extra_data: boxed_raw as u64,
        stack_size: 10 * 1024,
        ip: thread_start as *const () as usize,
    });
    let result = thread_syscall.invoke();
    let thread_id = match result {
        moondust_sys::syscall::Sysrets::SuccessWithVal(thread_id) => thread_id,
        _ => panic!("Spawn failure."),
    };

    JoinHandle {
        _result: my_result,
        _thread_id: thread_id,
    }
}

pub struct JoinHandle<T> {
    _result: Arc<UnsafeCell<Option<Result<T, ()>>>>,
    _thread_id: u64,
}
