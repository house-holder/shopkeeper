use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct Cents(u32);

impl Cents {
    fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cents = self.0;
        write!(f, "{}.{:02}", cents / 100, cents % 100)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct Grams(u32);

impl fmt::Display for Grams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let grams = self.0;
        if self.0 >= 908 {
            let pounds = (grams as f64 / 453.59237).ceil() as u32;
            write!(f, "{pounds}lb")
        } else if self.0 >= 57 {
            let ounces = (grams as f64 / 28.349523125).ceil() as u32;
            write!(f, "{ounces}oz")
        } else {
            write!(f, "{grams}g")
        }
    }
}

struct Item {
    name: String,
    id: u32,
    cost: Cents,
    weight: Grams,
}

// enum OrderStatus {
//     New { date_created: String },
//     Shipped { tracking: String },
//     Completed { date_delivered: String },
//     Canceled { reason: String },
//     Returned { reason: String },
// }

#[derive(Debug)]
struct OrderLine {
    item_id: u32,
    qty: u32,
}

struct Order {
    id: u32,
    // status: OrderStatus,
    cost: Cents,
    ship_weight: Grams,
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
        let input_grams = Grams(retry_read_u32("  Item weight (g): ")?);
        let input_qty = retry_read_u32("  Quantity: ")?;

        let new_item = Item {
            name: input_name,
            id: self.next_item_id,
            cost: Cents(input_cents),
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

    fn build_order(&mut self) -> io::Result<Option<Vec<OrderLine>>> {
        let mut order_qty: HashMap<u32, u32> = HashMap::new();

        loop {
            self.display();
            let mut ids: Vec<u32> = self.inventory.keys().copied().collect();
            ids.sort_unstable();

            let cmd = read_str("  > Select row # ('f' to finish, 'q' to quit): ")?;
            match cmd.as_str() {
                "f" => {
                    if order_qty.is_empty() {
                        eprintln!("Unable to complete order, no items have been added.");
                        continue;
                    }

                    let lines: Vec<OrderLine> = order_qty
                        .into_iter()
                        .map(|(item_id, qty)| OrderLine { item_id, qty })
                        .collect();

                    return Ok(Some(lines));
                }
                "q" => {
                    // canceling: restore inventory from order_qty
                    for (item_id, qty) in order_qty.iter() {
                        let _ = self.adjust_stock(*item_id, *qty as i32);
                    }
                    return Ok(None);
                }
                _ => {
                    // row number
                }
            }
            let row: usize = match cmd.parse() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("Invalid input.");
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
                }
            }
        }
    }

    fn commit_order(&mut self, lines: Vec<OrderLine>) -> Order {
        let mut order_cost: u64 = 0;
        let mut order_grams: u64 = 0;
        for l in &lines {
            let (item, _avail) = self.inventory.get(&l.item_id).expect("Line item not found");
            let qty_u64 = u64::from(l.qty);
            order_cost += u64::from(item.cost.as_u32()) * qty_u64;
            order_grams += u64::from(item.weight.0) * qty_u64;
        }
        let cost_u32: u32 = order_cost.try_into().expect("Failed to convert order_cost");
        let new_order = Order {
            id: self.next_order_id,
            // status: OrderStatus::New {
            //     date_created: "12DEC2025".to_string(),
            // },
            cost: Cents(cost_u32),
            ship_weight: Grams(
                order_grams
                    .try_into()
                    .expect("Failed to convert order_grams"),
            ),
            items: lines,
        };
        self.next_order_id += 1;

        new_order
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
            println!(" {id:06} | {:40} | ${:>9} | {qty:5}", item.name, item.cost);
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

    let item1 = Item {
        name: "36\" cyl packing kit".to_string(),
        id: 308113,
        cost: Cents(2299),
        weight: Grams(81),
    };
    let item2 = Item {
        name: "36\" cylinder housing".to_string(),
        id: 389120,
        cost: Cents(83500),
        weight: Grams(12613),
    };
    let item3 = Item {
        name: "Flat washer (5/16\", stainless)".to_string(),
        id: 210001,
        cost: Cents(8),
        weight: Grams(2),
    };
    let item4 = Item {
        name: "Bearing - conical, 0.875\"ID".to_string(),
        id: 992871,
        cost: Cents(3895),
        weight: Grams(925),
    };

    store.stock(item1, 12);
    store.stock(item2, 8);
    store.stock(item3, 203);
    store.stock(item4, 2);

    while store.inventory.len() < 4 {
        while let Err(e) = store.create_stock() {
            eprintln!("create_stock failed: {e}");
        }
    }

    if let Some(lines) = store.build_order()? {
        let order = store.commit_order(lines);

        for l in &order.items {
            let (item, _avail) = store
                .inventory
                .get(&l.item_id)
                .expect("Item is missing from inventory");
            let line_total = Cents(item.cost.as_u32() * l.qty);
            println!("  x{}  {}  ${}", l.qty, item.name, line_total);
        }
        println!("total=${} ship={}", order.cost, order.ship_weight);

        store.orders.push(order);
    } else {
        eprintln!("Order not completed.");
    }

    for o in &store.orders {
        println!("order #{} total=${} ship={}", o.id, o.cost, o.ship_weight);
    }

    Ok(())
}
