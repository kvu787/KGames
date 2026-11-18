// The process's GPU scheduling priority, which RunLightly lowers through the
// environment, so that a game the user is playing keeps its frames while
// coding agents' tests draw.

use std::sync::{Mutex, MutexGuard};

use graphics::{AdapterChoice, Gpu, GpuPriority, gpu_priority, set_gpu_priority};

// The priority and the environment are the process's, so these tests run
// one at a time.
fn process() -> MutexGuard<'static, ()> {
    static PROCESS: Mutex<()> = Mutex::new(());
    PROCESS.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn names_are_the_scheduler_s_classes() {
    assert_eq!(GpuPriority::from_name("Idle"), Some(GpuPriority::Idle));
    assert_eq!(GpuPriority::from_name("BelowNormal"), Some(GpuPriority::BelowNormal));
    assert_eq!(GpuPriority::from_name("Normal"), Some(GpuPriority::Normal));
    assert_eq!(GpuPriority::from_name("AboveNormal"), Some(GpuPriority::AboveNormal));
    assert_eq!(GpuPriority::from_name("High"), Some(GpuPriority::High));
    assert_eq!(GpuPriority::from_name("idle"), None);
    assert_eq!(GpuPriority::from_name(""), None);
}

#[test]
fn the_process_s_priority_can_be_lowered_and_put_back() {
    let _process = process();
    set_gpu_priority(GpuPriority::Idle).unwrap();
    assert_eq!(gpu_priority().unwrap(), GpuPriority::Idle);
    set_gpu_priority(GpuPriority::BelowNormal).unwrap();
    assert_eq!(gpu_priority().unwrap(), GpuPriority::BelowNormal);
    set_gpu_priority(GpuPriority::Normal).unwrap();
    assert_eq!(gpu_priority().unwrap(), GpuPriority::Normal);
}

#[test]
fn a_device_takes_the_environment_s_priority() {
    let _process = process();
    unsafe { std::env::set_var("KGAMES_GPU_PRIORITY", "Idle") };
    let gpu = Gpu::new(AdapterChoice::HighPerformance);
    unsafe { std::env::remove_var("KGAMES_GPU_PRIORITY") };
    gpu.unwrap();
    assert_eq!(gpu_priority().unwrap(), GpuPriority::Idle);
    set_gpu_priority(GpuPriority::Normal).unwrap();
}

#[test]
fn without_the_environment_a_device_leaves_the_priority_alone() {
    let _process = process();
    unsafe { std::env::remove_var("KGAMES_GPU_PRIORITY") };
    set_gpu_priority(GpuPriority::BelowNormal).unwrap();
    let gpu = Gpu::new(AdapterChoice::HighPerformance).unwrap();
    assert_eq!(gpu_priority().unwrap(), GpuPriority::BelowNormal);
    drop(gpu);
    set_gpu_priority(GpuPriority::Normal).unwrap();
}

#[test]
fn a_wrong_value_in_the_environment_is_an_error() {
    let _process = process();
    unsafe { std::env::set_var("KGAMES_GPU_PRIORITY", "Lowest") };
    let result = Gpu::new(AdapterChoice::HighPerformance);
    unsafe { std::env::remove_var("KGAMES_GPU_PRIORITY") };
    let error = result.err().expect("the device shouldn't be made");
    assert!(error.message().contains("KGAMES_GPU_PRIORITY must be"), "{error}");
}
