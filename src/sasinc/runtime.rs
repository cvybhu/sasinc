use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};
use std::time::Instant;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + 'static>>;

struct Task {
    future: RefCell<BoxFuture>,
    runtime: Rc<Runtime>,
}

impl Task {
    fn wake(self: Rc<Task>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Rc<Task>) {
        self.runtime.task_queue.borrow_mut().push_back(self.clone());
    }

    fn into_waker(self: Rc<Task>) -> Waker {
        // Why the hell can't I do Rc<dyn WakerTrait> ????
        return unsafe {
            Waker::from_raw(RawWaker::new(
                Rc::into_raw(self) as *const (),
                Self::waker_vtable(),
            ))
        };
    }

    fn waker_vtable() -> &'static RawWakerVTable {
        return &RawWakerVTable::new(
            Self::clone_rc_raw,
            Self::wake_rc_raw,
            Self::wake_by_ref_rc_raw,
            Self::drop_rc_raw,
        );
    }

    unsafe fn clone_rc_raw(data: *const ()) -> RawWaker {
        let rc: Rc<Task> = Rc::<Task>::from_raw(data as *const Task); // Recover Rc from pointer
        let result_rc: Rc<Task> = rc.clone();
        let _ = Rc::into_raw(rc); // don't drop the Rc
        return RawWaker::new(Rc::into_raw(result_rc) as *const (), Self::waker_vtable());
    }

    unsafe fn wake_rc_raw(data: *const ()) {
        let rc: Rc<Task> = Rc::<Task>::from_raw(data as *const Task); // Recover Rc from pointer
        rc.wake();
    }

    unsafe fn wake_by_ref_rc_raw(data: *const ()) {
        let rc: Rc<Task> = Rc::<Task>::from_raw(data as *const Task); // Recover Rc from pointer
        rc.wake_by_ref();
        Rc::into_raw(rc); // don't drop the Rc
    }

    unsafe fn drop_rc_raw(data: *const ()) {
        drop(Rc::<Task>::from_raw(data as *const Task));
    }
}

pub struct Runtime {
    task_queue: RefCell<VecDeque<Rc<Task>>>,
    waiting_timers: RefCell<BTreeMap<(Instant, u64), Waker>>,
    next_timer_id: Cell<u64>,
}

impl Runtime {
    pub fn new() -> Rc<Runtime> {
        return Rc::new(Runtime {
            task_queue: RefCell::new(VecDeque::new()),
            waiting_timers: RefCell::new(BTreeMap::new()),
            next_timer_id: Cell::new(0),
        });
    }

    pub fn spawn(self: &Rc<Self>, future: impl Future<Output = ()> + 'static) {
        self.task_queue.borrow_mut().push_back(Rc::new(Task {
            future: RefCell::new(Box::pin(future)),
            runtime: self.clone(),
        }));
    }

    pub fn run(self: &Rc<Self>) {
        while self.task_queue.borrow().len() != 0 || self.waiting_timers.borrow().len() != 0 {
            loop {
                let task: Rc<Task> = match self.task_queue.borrow_mut().pop_front() {
                    Some(task) => task,
                    None => break,
                };

                let waker: Waker = task.clone().into_waker();
                let _ = task
                    .future
                    .borrow_mut()
                    .as_mut()
                    .poll(&mut Context::from_waker(&waker));
            }

            // Nothing to do - wait until first timer expires
            let (timer_until, timer_id): (Instant, u64) =
                match self.waiting_timers.borrow().keys().next() {
                    Some(key) => *key,
                    None => continue,
                };

            let now = std::time::Instant::now();
            if now < timer_until {
                std::thread::sleep(timer_until.duration_since(now));
            }

            let timer_waker: Waker = self
                .waiting_timers
                .borrow_mut()
                .remove(&(timer_until, timer_id))
                .unwrap();
            timer_waker.wake();
        }
    }

    pub fn register_timer(&self, sleep_until_time: Instant, waker: Waker) {
        let cur_timer_id: u64 = self.next_timer_id.get();
        self.next_timer_id.set(cur_timer_id.wrapping_add(1));

        self.waiting_timers
            .borrow_mut()
            .insert((sleep_until_time, cur_timer_id), waker);
    }
}
