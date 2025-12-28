use std::collections::HashMap;
use std::io::{self, Write};

struct Item {
    name: String,
    id: u32,
    cost_cents: u32,
    weight_lbs: u32,
}

impl Item {}

enum OrderStatus {
    New { date_created: String },
    Canceled { reason: String },
    Shipped { tracking: String },
    Completed { date_delivered: String },
}

struct Order {
    status: OrderStatus,
    cost_cents: u32,
    shipped_lbs: u32,
    items: Vec<Item>,
}

struct Store {
    inventory: HashMap<u32, (Item, u32)>,
    orders: Vec<Order>,
    next_item_id: u32,
    next_order_id: u32,
}

impl Store {
    fn new() -> Store {
        Store {
            inventory: HashMap::new(),
            orders: Vec::new(),
            next_item_id: 1,
            next_order_id: 1,
        }
    }

    fn stock(&mut self, item: Item, quantity: u32) {
        self.inventory.insert(item.id, (item, quantity));
    }

    fn create_stock(&mut self) -> io::Result<()> {
        println!("Creating new stock item...");
        let input_name = read_str("  Item name: ")?;
        let input_cents = retry_read_u32("  Item price: ")?;
        let input_lbs = retry_read_u32("  Item weight (lbs): ")?;
        let input_qty = retry_read_u32("  Quantity: ")?;

        let new_item = Item {
            name: input_name,
            id: self.next_item_id,
            cost_cents: input_cents,
            weight_lbs: input_lbs,
        };

        self.stock(new_item, input_qty);
        self.next_item_id += 1;
        Ok(())
    }

    fn restock(&mut self, item_id: u32, qty_change: i32) {
        // accepts a positive or negative qty_change
    }

    fn display(&self) {
        let border: String = "-".repeat(76);
        println!(
            "{border}\n {:8}{:40}{:8}{:6}\n{border}",
            "ID", "Description", "Price", "Qty",
        );

        let mut items: Vec<_> = self.inventory.iter().collect();
        items.sort_by_key(|(id, _)| *id);

        for (id, (item, qty)) in items {
            println!(
                " {id:06}  {:34} ${:6}.{:02}  {qty:5}",
                item.name,
                item.cost_cents / 100,
                item.cost_cents % 100
            );
        }
    }
}

// generic helpers
fn read_str(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn read_u32(prompt: &str) -> io::Result<u32> {
    let s = read_str(prompt)?;
    s.parse::<u32>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
}

fn retry_read_u32(prompt: &str) -> io::Result<u32> {
    loop {
        match read_u32(prompt) {
            Ok(n) => return Ok(n),
            Err(e) if e.kind() == io::ErrorKind::InvalidInput => {
                eprintln!("Invalid input, try again.");
                continue;
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                eprintln!("Timeout");
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

fn main() -> io::Result<()> {
    println!("Inventory System Running.");
    let mut store = Store::new();

    store.create_stock()?;
    store.display();

    Ok(())
}
