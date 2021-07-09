use crate::{
    process_table::MyProcess,
    serial_println
};
use spin::Mutex;

struct MyScheduler {
    head : *mut MyProcess,
    tail : *mut MyProcess
}

#[allow(dead_code)]
impl MyScheduler {
    fn reset(&mut self) {
        self.head = 0x0 as *mut MyProcess;
        self.tail = 0x0 as *mut MyProcess;
        return;
    }

    fn get_head(&mut self) -> &mut MyProcess {
        unsafe{&mut (*self.head)}
    }

    fn get_tail(&mut self) -> &mut MyProcess {
        unsafe{&mut (*self.tail)}
    }

    fn pop(&mut self) -> (Option<&mut MyProcess>, &mut MyScheduler) {
        return if self.head == 0x0 as *mut MyProcess {
            (None, self)
        } else {
            let head_ref = unsafe { &mut (*self.head) };
            let next_ref = head_ref.get_next();
            match next_ref {
                Some(val) => {
                    self.head = val;
                },
                None => {
                    self.reset();
                },
            }
            (Some(head_ref), self)
        }
    }

    pub fn _yield(&mut self) {
        let x = self.pop();
        let option = x.0;
        let self_ref = x.1;
        if option.is_none() {
            return;
        }
        let yield_pt = option.unwrap();
        let resume_pt = crate::process_table::get_curr_process_table_mut();
//        serial_println!("{} -> {}", resume_pt.process_id, yield_pt.process_id);
        resume_pt.set_next(None);
        self_ref.resume(resume_pt);

        crate::process_table::set_next_process(yield_pt);
        crate::process_table::process_switch_to();
    }

    pub fn resume(&mut self, proc : &mut MyProcess) {
        if self.head == 0x0 as *mut MyProcess {
            proc.set_next(None);
            self.head = proc;
            self.tail = proc;
        } else {
            self.get_tail().set_next(Some(proc));
            self.tail = proc;
        }
    }

    pub fn terminate(&mut self, _proc : &mut MyProcess) {

    }
}

static mut SYSTEM_SCHEDULER : MyScheduler = MyScheduler {
    head : 0x0 as *mut MyProcess,
    tail : 0x0 as *mut MyProcess
};

static mut SCHEDULER_MUTEX : Mutex<bool> = Mutex::new(true);

pub fn _yield() {
    unsafe {
        SCHEDULER_MUTEX.lock();
        SYSTEM_SCHEDULER._yield();
    }
}

pub fn resume(proc : &mut MyProcess) {
    unsafe {
        SCHEDULER_MUTEX.lock();
        SYSTEM_SCHEDULER.resume(proc);
    }
}