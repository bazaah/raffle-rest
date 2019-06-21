use {
    rand::{
        distributions::{Distribution, Uniform},
        thread_rng as rng,
    },
    serde::Serialize,
    serde_json::{json, value::Value as jVal},
    std::{collections::BTreeMap, fmt},
};

// External interface object which manages the Ticket(s)
// and ensures all Ticket(s) have a unique ID
pub struct Raffle {
    count: u64,
    tickets: BTreeMap<u64, Ticket>,
}

impl Raffle {
    // Creates the base object -- used by rocket to generate managed state
    pub fn instantiate() -> Self {
        let count = 0u64;
        let tickets: BTreeMap<u64, Ticket> = BTreeMap::new();
        Raffle { count, tickets }
    }

    // Generates a new Ticket and returns its ID
    pub fn new_ticket(&mut self, lines: Option<u64>) -> u64 {
        self.count += 1;

        if self.tickets.contains_key(&self.count) {
            self.count = self.find_unused_key()
        }

        // If a user provided N lines use them
        // otherwise use default [10]
        let ticket = match lines {
            Some(lines) => Ticket::from(lines),
            None => Ticket::new(),
        };

        self.tickets.insert(self.count, ticket);
        eprintln!("{:?}", &self.tickets);
        self.count
    }

    // Returns a user defined Ticket if it exists, or an error if it doesn't
    pub fn get_ticket(&self, id: u64) -> Result<jVal, ErrorKind> {
        match self.tickets.get(&id) {
            Some(ticket) => Ok(json!({"id": id, "lines": ticket.eval_list()})),
            None => {
                let err = Err(ErrorKind::TicketNotFound(id));
                eprintln!("in models {:?}", err);
                err
            }
        }
    }

    // Appends N [additional] number of lines to a user defined Ticket,
    // or returns an error if the ID doesn't exist
    pub fn append_ticket(&mut self, id: u64, additional: u64) -> Result<(), ErrorKind> {
        match self.tickets.get_mut(&id) {
            Some(ticket) => Ok(ticket.append(additional)),
            None => {
                let err = Err(ErrorKind::TicketNotFound(id));
                eprintln!("in models {:?}", err);
                err
            }
        }
    }

    // Returns the entire list of tickets as Json
    pub fn get_ticket_list(&self) -> jVal {
        let json: jVal = self
            .tickets
            .iter()
            .map(|(idx, ticket)| {
                json!({
                    "id": idx,
                    "lines": ticket.eval_list()
                })
            })
            .collect();

        json
    }

    // Uses up a Ticket and returns a rough estimate of how lucky the user was,
    // or returns an error if the ID doesn't exist
    pub fn evaluate_ticket(&mut self, id: u64) -> Result<jVal, ErrorKind> {
        match self.tickets.remove(&id) {
            Some(ticket) => {
                let list = ticket.eval_list();
                let sum: u64 = list.iter().map(|i| *i as u64).sum();
                let score = sum / list.len() as u64;
                let response = match score {
                    _n @ 0 => format_args!("you get nothing; good day sir!"),
                    _n @ 1..=3 => format_args!("slightly better than a hostel shower!"),
                    _n @ 4..=7 => format_args!("you're one of today's lucky 10,000!"),
                    _n @ 8..=9 => format_args!("almost enough for a mediocre pizza!"),
                    _ => format_args!("ding ding ding, you won the imaginary jackpot!"),
                };
                Ok(json!(format!(
                    "For ticket {}, your score was {}... {}",
                    id, score, response
                )))
            }
            None => Err(ErrorKind::TicketNotFound(id)),
        }
    }

    // Internal function for finding the next unique ID
    fn find_unused_key(&self) -> u64 {
        (self.count..)
            .filter(|k| !self.tickets.contains_key(k))
            .take(1)
            .sum()
    }
}

// Internal representation of a Ticket
#[derive(Clone, Serialize, Debug)]
struct Ticket {
    line_list: Vec<Line>,
}

impl Ticket {
    // Creates a Ticket with the default number of Lines [10]
    // uses thread-specific system entropy for its RNG
    fn new() -> Self {
        let line_list = (0..10)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .map(|rand| Line::from(rand))
            .collect::<Vec<Line>>();

        Ticket { line_list }
    }

    // Creates a Ticket with a custom number of Lines [lines]
    // uses thread-specific system entropy for its RNG
    fn from(lines: u64) -> Self {
        let line_list = (0..lines)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .map(|seed| Line::from(seed))
            .collect::<Vec<Line>>();

        Ticket { line_list }
    }

    // Appends N [additional] lines to an existing ticket
    fn append(&mut self, additional: u64) {
        (0..additional)
            .scan((rng(), Uniform::from(0..3)), |(s, r), _| {
                Some((r.sample(s), r.sample(s), r.sample(s)))
            })
            .for_each(|seed| self.line_list.push(Line::from(seed)))
    }

    // Computes the output for all Lines in a Ticket
    fn eval_list(&self) -> Vec<u8> {
        self.line_list
            .iter()
            .inspect(|line| eprintln!("{:?}", line))
            .map(|line| line.eval_line())
            .collect::<Vec<u8>>()
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, line) in self.line_list.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", line)?;
        }
        write!(f, "]")
    }
}

// Named tuple which holds 3 numbers between 0 and 2: [0,1,2]
#[derive(Debug, Clone, Copy, Serialize)]
struct Line(u8, u8, u8);

impl Line {
    // A Line can only be generated by a Ticket
    fn from((x, y, z): (u8, u8, u8)) -> Self {
        Line(x, y, z)
    }

    // Computes a Line's output based on the given rules
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

// Error kind(s) used by Raffle
#[derive(Debug, Clone)]
pub enum ErrorKind {
    TicketNotFound(u64),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::TicketNotFound(id) => write!(f, "Ticket id: {} doesn't exist", id),
        }
    }
}
