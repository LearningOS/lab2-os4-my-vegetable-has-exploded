//! Process management syscalls

use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{translated_ptr, VirtAddr};
use crate::task::{
    current_user_token, exit_current_and_run_next, get_syscall_record, get_time_interval, mmap,
    suspend_current_and_run_next, unmap, TaskStatus,
};
use crate::timer::get_time_us;
use core::convert::TryInto;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let ts = translated_ptr(current_user_token(), _ts);
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

fn check_mmap_port(port: usize) -> bool {
    if (port & (!0x07) != 0) || (port & 0x7 == 0) {
        return false;
    }
    true
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if check_mmap_port(_port) == false {
        return -1;
    }
    let virt_start = VirtAddr::from(_start);
    if virt_start.page_offset() != 0 {
        return -1;
    }
    if _len <= 0 {
        return -1;
    }
    let virt_end = VirtAddr::from(_start + _len);
    mmap(virt_start, virt_end, _port)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    let virt_start = VirtAddr::from(_start);
    if virt_start.page_offset() != 0 {
        return -1;
    }
    if _len <= 0 {
        return -1;
    }
    let virt_end = VirtAddr::from(_start + _len);
    unmap(virt_start, virt_end)
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let ti = translated_ptr(current_user_token(), ti);
    unsafe {
        *ti = TaskInfo {
            status: TaskStatus::Running,
            syscall_times: get_syscall_record().try_into().unwrap(),
            time: get_time_interval(),
        }
    }
    0
}
