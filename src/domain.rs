use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Cents(u32);

impl Cents {
    pub fn new(cents: u32) -> Self {
        Self(cents)
    }

    pub fn as_u32(self) -> u32 {
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
pub struct Grams(u32);

impl Grams {
    pub fn new(grams: u32) -> Self {
        Self(grams)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Grams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let grams = self.0;
        if grams >= 908 {
            let pounds = (grams as f64 / 453.59237).ceil() as u32;
            write!(f, "{pounds}lb")
        } else if grams >= 57 {
            let ounces = (grams as f64 / 28.349523125).ceil() as u32;
            write!(f, "{ounces}oz")
        } else {
            write!(f, "{grams}g")
        }
    }
}

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub id: u32,
    pub cost: Cents,
    pub weight: Grams,
}

#[derive(Debug)]
pub struct OrderLine {
    pub item_id: u32,
    pub qty: u32,
}

#[derive(Debug)]
pub struct Order {
    pub id: u32,
    pub cost: Cents,
    pub ship_weight: Grams,
    pub items: Vec<OrderLine>,
}

pub struct Store {
    inventory: HashMap<u32, (Item, u32)>,
    orders: Vec<Order>,
    next_item_id: u32,
    next_order_id: u32,
}

impl Store {
    pub fn new() -> Store {
        Store {
            inventory: HashMap::new(),
            orders: Vec::new(),
            next_item_id: 1,
            next_order_id: 1,
        }
    }

    pub fn inventory_len(&self) -> usize {
        self.inventory.len()
    }

    pub fn inventory_ids_sorted(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.inventory.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    pub fn inventory_get(&self, item_id: u32) -> Option<(&Item, u32)> {
        self.inventory.get(&item_id).map(|(item, qty)| (item, *qty))
    }

    pub fn stock(&mut self, item: Item, quantity: u32) {
        self.inventory.insert(item.id, (item, quantity));
    }

    pub fn stock_new(&mut self, name: String, cost: Cents, weight: Grams, quantity: u32) -> u32 {
        let id = self.next_item_id;
        let item = Item {
            name,
            id,
            cost,
            weight,
        };
        self.stock(item, quantity);
        self.next_item_id += 1;
        id
    }

    pub fn adjust_stock(&mut self, item_id: u32, qty_change: i32) -> Result<u32, String> {
        let (_item, qty) = self
            .inventory
            .get_mut(&item_id)
            .ok_or_else(|| format!("Unknown id: {item_id}"))?;

        let new_qty = (*qty as i64) + (qty_change as i64);
        if new_qty < 0 {
            return Err(format!("Not enough stock (ID: {item_id})"));
        }

        *qty = new_qty as u32;
        Ok(*qty)
    }

    pub fn commit_order(&mut self, lines: Vec<OrderLine>) -> Order {
        let mut order_cost: u64 = 0;
        let mut order_grams: u64 = 0;

        for l in &lines {
            let (item, _avail) = self.inventory.get(&l.item_id).expect("Line item not found");
            let qty_u64 = u64::from(l.qty);
            order_cost += u64::from(item.cost.as_u32()) * qty_u64;
            order_grams += u64::from(item.weight.as_u32()) * qty_u64;
        }

        let cost_u32: u32 = order_cost.try_into().expect("order cost too large");
        let grams_u32: u32 = order_grams.try_into().expect("order weight too large");

        let new_order = Order {
            id: self.next_order_id,
            cost: Cents(cost_u32),
            ship_weight: Grams(grams_u32),
            items: lines,
        };

        self.next_order_id += 1;
        new_order
    }

    pub fn orders(&self) -> &[Order] {
        &self.orders
    }

    pub fn push_order(&mut self, order: Order) {
        self.orders.push(order);
    }
}
