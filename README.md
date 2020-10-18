# Sasinc
Sasin inspired async runtime  
  
This is a single threaded async runtime written from scratch to learn more about async/await in Rust  
Supports timers and asynchronous money collecting/withdrawal  
See example:  
```rust
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
            "Sasin here - I spent {} and got {} usable ballots!",
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
```

### Run
To run install Rust and execute `cargo run`