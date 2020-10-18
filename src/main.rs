mod money;
mod sasinc;

use money::Money;
use std::rc::Rc;
use std::time::Duration;

type GovermentBudget = sasinc::money_pool::MoneyPool;

struct Taxpayer {
    salary: Money,
    time_between_payouts: Duration,
    goverment_budget: Rc<GovermentBudget>,
    async_runtime: Rc<sasinc::runtime::Runtime>, // Sadly that's needed, in tokio it's global
}

struct Sasin {
    goverment_budget: Rc<GovermentBudget>,
    async_runtime: Rc<sasinc::runtime::Runtime>,
}

struct Ballot {}

impl Taxpayer {
    async fn go(&self) {
        for _ in 0..5 {
            let my_money: Money = self.work().await;
            let tax_to_pay: Money = my_money;
            self.goverment_budget.add_money(tax_to_pay);
        }
    }

    async fn work(&self) -> Money {
        sasinc::timer::sleep_for(self.time_between_payouts, self.async_runtime.clone()).await;
        return self.salary;
    }
}

impl Sasin {
    async fn go(&mut self) {
        println!("Sasin here - I want money from the goverment!");

        let seventy_mil: Money = self.goverment_budget.take_money(70_000_000).await;

        println!("Sasin here - I have money from goverment!");

        let ballots: Vec<Ballot> = self.buy_ballots(seventy_mil).await;

        println!(
            "Sasin here - I spend {} money and got {} usable ballots!",
            seventy_mil,
            ballots.len()
        );
    }

    async fn buy_ballots(&mut self, money: Money) -> Vec<Ballot> {
        let sleep_duration = std::time::Duration::from_secs(1);
        sasinc::timer::sleep_for(sleep_duration, self.async_runtime.clone()).await;
        return vec![];
    }
}

fn main() {
    let runtime: Rc<sasinc::runtime::Runtime> = sasinc::runtime::Runtime::new();
    let goverment_budget: Rc<GovermentBudget> = Rc::new(GovermentBudget::new());

    // Spawn sasin
    let mut sasin: Sasin = Sasin {
        goverment_budget: goverment_budget.clone(),
        async_runtime: runtime.clone(),
    };
    runtime.spawn(async move {
        sasin.go().await;
    });

    // Spawn taxpayers
    for _ in 0..10000 {
        let taxpayer = Taxpayer {
            salary: 2600,
            time_between_payouts: Duration::from_secs(1),
            goverment_budget: goverment_budget.clone(),
            async_runtime: runtime.clone(),
        };

        runtime.spawn(async move {
            taxpayer.go().await;
        })
    }

    // Go
    runtime.run();
}
