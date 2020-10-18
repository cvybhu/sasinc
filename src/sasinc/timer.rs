use super::runtime::Runtime;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub async fn sleep_for(time_duration: Duration, runtime: Rc<Runtime>) {
    let time_instant: Instant = Instant::now() + time_duration;
    return sleep_until(time_instant, runtime).await;
}

pub async fn sleep_until(time_instant: Instant, runtime: Rc<Runtime>) {
    return TimerFuture::new(time_instant, runtime).await;
}

struct TimerFuture {
    sleep_until: Instant,
    runtime: Rc<Runtime>,
}

impl TimerFuture {
    fn new(sleep_until: Instant, runtime: Rc<Runtime>) -> TimerFuture {
        return TimerFuture {
            sleep_until,
            runtime,
        };
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.sleep_until {
            return Poll::Ready(());
        } else {
            self.runtime
                .register_timer(self.sleep_until, context.waker().clone());
            return Poll::Pending;
        }
    }
}
