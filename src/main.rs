mod domain;
mod storage;
mod ui;

use std::io;

use crate::domain::{Cents, Grams, Item, Store};

fn main() -> io::Result<()> {
    let mut store = Store::new();

    let item1 = Item {
        name: "36\" cyl packing kit".to_string(),
        id: 308113,
        cost: Cents::new(2299),
        weight: Grams::new(81),
    };
    let item2 = Item {
        name: "36\" cylinder housing".to_string(),
        id: 389120,
        cost: Cents::new(83500),
        weight: Grams::new(12613),
    };
    let item3 = Item {
        name: "Flat washer (5/16\", stainless)".to_string(),
        id: 210001,
        cost: Cents::new(8),
        weight: Grams::new(2),
    };
    let item4 = Item {
        name: "Bearing - conical, 0.875\"ID".to_string(),
        id: 992871,
        cost: Cents::new(3895),
        weight: Grams::new(925),
    };

    store.stock(item1, 12);
    store.stock(item2, 8);
    store.stock(item3, 203);
    store.stock(item4, 2);

    while store.inventory_len() < 4 {
        while let Err(e) = ui::create_stock(&mut store) {
            eprintln!("create_stock failed: {e}");
        }
    }

    if let Some(lines) = ui::build_order(&mut store)? {
        let order = store.commit_order(lines);
        ui::print_receipt(&store, &order);
        store.push_order(order);
    } else {
        eprintln!("Order not completed.");
    }

    for o in store.orders() {
        println!("order #{} total=${} ship={}", o.id, o.cost, o.ship_weight);
    }

    Ok(())
}
