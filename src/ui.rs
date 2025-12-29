use std::io::{self, Write};

use crate::domain::{Cents, Grams, Order, OrderLine, Store};

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
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn create_stock(store: &mut Store) -> io::Result<()> {
    println!("Creating new stock item...");
    let input_name = read_str("  Item name: ")?;
    let input_cents = retry_read_u32("  Item price (cents): ")?;
    let input_grams = retry_read_u32("  Item weight (g): ")?;
    let input_qty = retry_read_u32("  Quantity: ")?;

    store.stock_new(
        input_name,
        Cents::new(input_cents),
        Grams::new(input_grams),
        input_qty,
    );

    Ok(())
}

pub fn display(store: &Store) {
    let border: String = "-".repeat(72);
    println!(
        "{border}\n {:6} | {:40} |  {:9} | {:5}\n{border}",
        "ID#", "Description", "Unit Cost", "Avail",
    );

    for id in store.inventory_ids_sorted() {
        let (item, qty) = store
            .inventory_get(id)
            .expect("inventory id list out of sync");
        println!(" {id:06} | {:40} | ${:>9} | {qty:5}", item.name, item.cost);
    }
}

pub fn build_order(store: &mut Store) -> io::Result<Option<Vec<OrderLine>>> {
    let mut order_qty: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();

    loop {
        display(store);
        let ids = store.inventory_ids_sorted();

        let cmd = read_str("  > Select row # ('f' to finish, 'q' to quit): ")?;
        match cmd.as_str() {
            "f" => {
                if order_qty.is_empty() {
                    eprintln!("Unable to complete order, no items have been added.");
                    continue;
                }

                let mut lines: Vec<OrderLine> = order_qty
                    .into_iter()
                    .map(|(item_id, qty)| OrderLine { item_id, qty })
                    .collect();
                lines.sort_by_key(|l| l.item_id);
                return Ok(Some(lines));
            }
            "q" => {
                for (item_id, qty) in order_qty.iter() {
                    let _ = store.adjust_stock(*item_id, *qty as i32);
                }
                return Ok(None);
            }
            _ => {}
        }

        let row: usize = match cmd.parse() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Enter a row number, 'f', or 'q'.");
                continue;
            }
        };

        if row >= ids.len() {
            eprintln!("Row out of range.");
            continue;
        }

        let item_id = ids[row];
        let qty = retry_read_u32("  > Qty: ")?;

        match store.adjust_stock(item_id, -(qty as i32)) {
            Ok(_new_avail) => {
                *order_qty.entry(item_id).or_insert(0) += qty;
            }
            Err(msg) => {
                eprintln!("{msg}");
            }
        }
    }
}

pub fn print_receipt(store: &Store, order: &Order) {
    for l in &order.items {
        let (item, _avail) = store
            .inventory_get(l.item_id)
            .expect("Item is missing from inventory");
        let line_total = Cents::new(item.cost.as_u32() * l.qty);
        println!("  x{}  {}  ${}", l.qty, item.name, line_total);
    }
    println!("total=${} ship={}", order.cost, order.ship_weight);
}
