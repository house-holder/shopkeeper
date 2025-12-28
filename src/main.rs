#![allow(dead_code)]
use std::collections::HashMap;
use std::io::{self, Write};

/* weight constants
28.349523125 //grams per oz
453.59237    // grams per lb
*/

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Weight(f64);

// enum Weight {
//     Pounds(u32),
//     Ounces(u32),
//     Grams(u32),
// }
//
impl Weight {
    fn to_grams(&self) -> u32 {
        match *self {
            Weight::Grams(g) => g,
            Weight::Ounces(oz) => oz * 28,
            Weight::Pounds(lb) => lb * 453,
        }
    }
}

struct Item {
    name: String,
    id: u32,
    cost_cents: u32,
    weight: Weight,
}

fn price_str(cost: u32) -> String {
    format!("{}.{:02}", cost / 100, cost % 100)
}

enum OrderStatus {
    New { date_created: String },
    Shipped { tracking: String },
    Completed { date_delivered: String },
    Canceled { reason: String },
    Returned { reason: String },
}

struct OrderLine {
    item_id: u32,
    qty: u32,
}

struct Order {
    status: OrderStatus,
    cost_cents: u32,
    shipped_weight: Weight,
    items: Vec<OrderLine>,
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
        let input_grams = Weight::Grams(retry_read_u32("  Item weight (g): ")?);
        let input_qty = retry_read_u32("  Quantity: ")?;

        let new_item = Item {
            name: input_name,
            id: self.next_item_id,
            cost_cents: input_cents,
            weight: input_grams,
        };

        self.stock(new_item, input_qty);
        self.next_item_id += 1;
        Ok(())
    }

    // accepts a positive or negative change to modify stock levels
    fn adjust_stock(&mut self, item_id: u32, qty_change: i32) -> Result<u32, String> {
        let (_item, qty) = self
            .inventory
            .get_mut(&item_id)
            .ok_or_else(|| format!("Unknown id: {item_id}"))?;
        let new_qty = (*qty as i64) + (qty_change as i64);
        if new_qty < 0 {
            // NOTE: possibly change this to reset stock to zero
            return Err(format!("Not enough stock (ID: {item_id})"));
        }
        *qty = new_qty as u32;

        Ok(*qty)
    }

    fn build_order(&mut self) -> io::Result<Vec<OrderLine>> {
        let mut order_qty: HashMap<u32, u32> = HashMap::new();

        loop {
            self.display();

            let mut ids: Vec<u32> = self.inventory.keys().copied().collect();
            ids.sort_unstable();

            let cmd = read_str("  > Select row # ('q' to finish): ")?;
            if cmd == "q" {
                break;
            }

            let row: usize = match cmd.parse() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("Enter a row number or 'q'.");
                    continue;
                }
            };

            if row >= ids.len() {
                eprintln!("Row out of range.");
                continue;
            }

            let item_id = ids[row];
            let qty = retry_read_u32("  > Qty: ")?;

            match self.adjust_stock(item_id, -(qty as i32)) {
                Ok(_new_avail) => {
                    *order_qty.entry(item_id).or_insert(0) += qty;
                }
                Err(msg) => {
                    eprintln!("{msg}");
                    continue;
                }
            }
        }

        let mut lines: Vec<OrderLine> = order_qty
            .into_iter()
            .map(|(item_id, qty)| OrderLine { item_id, qty })
            .collect();
        lines.sort_by_key(|l| l.item_id);

        Ok(lines)
    }

    fn commit_order(&mut self, _lines: Vec<OrderLine>) -> Order {
        todo!("commit_order: compute totals, assign id, push to self.orders")
    }

    fn display(&self) {
        let border: String = "-".repeat(72);
        println!(
            "{border}\n {:6} | {:40} |  {:9} | {:5}\n{border}",
            "ID#", "Description", "Unit Cost", "Avail",
        );

        let mut items: Vec<_> = self.inventory.iter().collect();
        items.sort_by_key(|(id, _)| *id);

        for (id, (item, qty)) in items {
            println!(
                " {id:06} | {:40} | ${:>9} | {qty:5}",
                item.name,
                price_str(item.cost_cents),
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
    let mut store = Store::new();

    // store.create_stock()?;
    let item1 = Item {
        name: "36\" cyl packing kit".to_string(),
        id: 308113,
        cost_cents: 2299,
        weight: Weight::Grams(81),
    };
    let item2 = Item {
        name: "36\" cylinder housing".to_string(),
        id: 389120,
        cost_cents: 83500,
        weight: Weight::Grams(12613),
    };
    let item3 = Item {
        name: "Flat washer (5/16\", stainless)".to_string(),
        id: 210001,
        cost_cents: 8,
        weight: Weight::Grams(2),
    };
    let item4 = Item {
        name: "Bearing - conical, 0.875\"ID".to_string(),
        id: 992871,
        cost_cents: 3895,
        weight: Weight::Grams(925),
    };

    store.stock(item1, 12);
    store.stock(item2, 8);
    store.stock(item3, 203);
    store.stock(item4, 2);

    store.display();

    Ok(())
}
