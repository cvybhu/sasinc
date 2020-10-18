use crate::money::Money;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

pub struct MoneyPool {
    money: Cell<Money>,
    take_requests: RefCell<VecDeque<Rc<RefCell<MoneyTakeRequest>>>>,
}

impl MoneyPool {
    pub fn new() -> MoneyPool {
        return MoneyPool {
            money: Cell::new(0),
            take_requests: RefCell::new(VecDeque::new()),
        };
    }

    pub fn add_money(&self, added_money: Money) {
        self.money.set(self.money.get() + added_money);

        loop {
            let request_amount: Money = match self.take_requests.borrow().front() {
                Some(request) => request.borrow().amount,
                None => break,
            };

            if request_amount > self.money.get() {
                break;
            }

            let take_request: Rc<RefCell<MoneyTakeRequest>> =
                self.take_requests.borrow_mut().pop_front().unwrap();

            self.money.set(self.money.get() - request_amount);
            take_request.borrow_mut().fulfilled = true;

            let waker_to_call: Waker = match take_request.borrow_mut().waker.take() {
                Some(waker) => waker,
                None => continue,
            };

            waker_to_call.wake();
        }
    }

    pub async fn take_money(&self, needed_money: Money) -> Money {
        let take_request = Rc::new(RefCell::new(MoneyTakeRequest {
            amount: needed_money,
            fulfilled: false,
            waker: None,
        }));

        self.take_requests
            .borrow_mut()
            .push_back(take_request.clone());
        return MoneyTakeFuture::new(take_request).await;
    }
}

struct MoneyTakeRequest {
    amount: Money,
    fulfilled: bool,
    waker: Option<Waker>,
}

struct MoneyTakeFuture {
    request: Rc<RefCell<MoneyTakeRequest>>,
}

impl MoneyTakeFuture {
    fn new(take_request: Rc<RefCell<MoneyTakeRequest>>) -> MoneyTakeFuture {
        return MoneyTakeFuture {
            request: take_request,
        };
    }
}

impl Future for MoneyTakeFuture {
    type Output = Money;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.request.borrow().fulfilled {
            return Poll::Ready(self.request.borrow().amount);
        } else {
            self.request.borrow_mut().waker = Some(context.waker().clone());
            return Poll::Pending;
        }
    }
}
