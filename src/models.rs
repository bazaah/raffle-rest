use {
    serde::Serialize,
    rand::{
        distributions::{Distribution, Uniform},
        thread_rng as rng,
    },
    serde_json::{json, value::Value as jVal},
    std::fmt,
};

pub struct Raffle {
    tickets: Vec<Ticket>,
}

impl Raffle {
    pub fn instantiate() -> Self {
        let tickets: Vec<Ticket> = Vec::new();
        Raffle { tickets }
    }

    pub fn new_ticket(&mut self) -> Result<(), ()> {
        match self.tickets.len() == usize::max_value() {
            false => {
                self.tickets.push(Ticket::new());
                Ok(())
            }
            true => Err(()),
        }
    }

    pub fn new_ticket_from(&mut self, lines: u64) -> Result<(), ()> {
        match self.tickets.len() == usize::max_value() {
            false => {
                self.tickets.push(Ticket::from(lines));
                Ok(())
            }
            true => Err(()),
        }
    }

    pub fn get_ticket_list(&self) -> Vec<jVal> {
        let json: Vec<jVal> = self
            .tickets
            .iter()
            .enumerate()
            .map(|(idx, ticket)| {
                json!({
                    "id": idx,
                    "lines": ticket.eval_list()
                })
            })
            .collect();

        json
    }
}

#[derive(Clone, Serialize)]
struct Ticket {
    line_list: Vec<Line>,
}

impl Ticket {
    fn new() -> Self {
        let line_list = (0..10)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .map(|rand| Line::from(rand))
            .collect::<Vec<Line>>();

        Ticket { line_list }
    }

    fn from(lines: u64) -> Self {
        let line_list = (0..lines)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .map(|seed| Line::from(seed))
            .collect::<Vec<Line>>();

        Ticket { line_list }
    }

    fn append(&mut self, additional: u64) {
        (0..additional)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .for_each(|seed| self.line_list.push(Line::from(seed)))
    }

    fn eval_list(&self) -> Vec<u8> {
        self.line_list
            .iter()
            .inspect(|line| eprintln!("{:?}", line))
            .map(|line| line.eval_line())
            .collect::<Vec<u8>>()
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct Line(u8, u8, u8);

impl Line {
    fn from((x, y, z): (u8, u8, u8)) -> Self {
        Line(x, y, z)
    }

    fn eval_line(&self) -> u8 {
        match (self.0, self.1, self.2) {
            // Ordered by rule priority
            // Avoids situations where ex. (2,0,0) satisfies
            // 2 rules: |x+y+z == 2| & |x!=y && x!=z|
            (x, y, z) if x + y + z == 2 => 10,
            (x, y, z) if x == y && y == z => 5,
            (x, y, z) if x != y && x != z => 1,
            (_, _, _) => 0,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|{}|", self.eval_line())
    }
}
