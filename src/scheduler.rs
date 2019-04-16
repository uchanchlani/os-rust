use crate::process_table::MyProcess;

struct MyScheduler {
    head : *mut MyProcess,
    tail : *mut MyProcess
}

#[allow(dead_code)]
impl MyScheduler {
    fn new() -> MyScheduler {
        MyScheduler {
            head : 0x0 as *mut MyProcess,
            tail : 0x0 as *mut MyProcess
        }

    }

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

    fn pop(&mut self) -> Option<&mut MyProcess> {
        return if self.head == 0x0 as *mut MyProcess {
            None
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
            Some(head_ref)
        }
    }

    pub fn _yield(&mut self) {

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
