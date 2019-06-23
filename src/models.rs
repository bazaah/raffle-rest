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
        self.count
    }

    // Returns a user defined Ticket if it exists, or an error if it doesn't
    pub fn get_ticket(&self, id: u64) -> Result<jVal, ErrorKind> {
        match self.tickets.get(&id) {
            Some(ticket) => Ok(json!({"id": id, "lines": ticket.eval_list()})),
            None => {
                let err = Err(ErrorKind::TicketNotFound(id));
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
                Ok(Raffle::generate_response(id, score))
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

    fn generate_response(id: u64, score: u64) -> jVal {
        let response = match score {
            _n @ 0 => format_args!("you get nothing; good day sir!"),
            _n @ 1..=3 => format_args!("slightly better than a hostel shower!"),
            _n @ 4..=7 => format_args!("you're one of today's lucky 10,000!"),
            _n @ 8..=9 => format_args!("almost enough for a mediocre pizza!"),
            _ => format_args!("ding ding ding, you won the imaginary jackpot!"),
        };
        json!(format!(
            "For ticket {}, your score was {}... {}",
            id, score, response
        ))
    }
}

// Internal representation of a Ticket
#[derive(Clone, Serialize, Debug, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
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

/*
Code
-------------------------------------------------------------------------------
Tests
*/

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    macro_rules! static_ticket {
        () => {{
            let line_list = LINE_SEED_VALUES
                .iter()
                .map(|i| Line::from(*i))
                .collect::<Vec<Line>>();
            let ticket = Ticket { line_list };
            ticket
        }};
    }

    // ErrorKind tests
    #[test]
    fn TicketNotFound_display() {
        assert_eq!(
            format!("{}", ErrorKind::TicketNotFound(42)),
            "Ticket id: 42 doesn't exist"
        )
    }

    // Line tests

    #[test]
    fn Line_from() {
        LINE_SEED_VALUES
            .iter()
            .for_each(|seed| assert_eq!(Line::from(*seed), Line(seed.0, seed.1, seed.2)));
    }

    #[test]
    fn Line_eval_line() {
        LINE_SEED_VALUES
            .iter()
            .zip(LINE_EVAL_VALUES.iter())
            .map(|(seed, value)| (Line::from(*seed).eval_line(), *value))
            .for_each(|(eval, value)| assert_eq!(eval, value))
    }

    #[test]
    fn Line_display() {
        let line = Line::from(LINE_SEED_VALUES[0]);
        let evaluated = LINE_EVAL_VALUES[0];

        assert_eq!(format!("{}", line), format!("|{}|", evaluated));
    }

    // Ticket tests
    #[test]
    fn Ticket_new() {
        let ticket = Ticket::new();
        let ticket2 = Ticket::new();
        // It is theoretically possible for this test to fail due the inherent randomness;
        // therefore check against a third rand generation, before failing the test
        assert!(ticket != ticket2 || ticket != Ticket::new())
    }

    #[test]
    fn Ticket_from() {
        (1..=10).for_each(|lines_num| {
            assert_eq!(Ticket::from(lines_num).line_list.len(), lines_num as usize)
        })
    }

    #[test]
    fn Ticket_append() {
        let start_size = 10;
        let additional_lines = 7;
        let end_size = 17;
        assert_eq!(start_size + additional_lines, end_size);

        let mut ticket = Ticket::from(start_size);
        ticket.append(additional_lines);
        assert_eq!(ticket.line_list.len(), end_size as usize);
    }

    #[test]
    fn Ticket_eval_list() {
        let line_list: Vec<Line> = LINE_SEED_VALUES.iter().map(|i| Line::from(*i)).collect();
        let ticket = Ticket { line_list };
        assert_eq!(
            ticket.eval_list(),
            LINE_EVAL_VALUES.iter().map(|i| *i).collect::<Vec<u8>>()
        );
    }
    #[test]
    fn Ticket_display() {
        let line_list: Vec<Line> = LINE_SEED_VALUES
            .iter()
            .take(10)
            .map(|i| Line::from(*i))
            .collect();
        let ticket = Ticket { line_list };
        let evals: Vec<u8> = LINE_EVAL_VALUES.iter().take(10).map(|i| *i).collect();
        assert_eq!(ticket.line_list.len(), evals.len());

        assert_eq!(
            format!("{}", ticket),
            format!(
                "[|{}| |{}| |{}| |{}| |{}| |{}| |{}| |{}| |{}| |{}|]",
                evals[0],
                evals[1],
                evals[2],
                evals[3],
                evals[4],
                evals[5],
                evals[6],
                evals[7],
                evals[8],
                evals[9],
            )
        )
    }

    // Raffle tests
    #[test]
    fn Raffle_instantiate() {
        let base_raffle = Raffle::instantiate();

        assert!(base_raffle.count == 0 && base_raffle.tickets.is_empty());
    }

    #[test]
    fn Raffle_new_ticket_default() {
        let mut raffle = Raffle::instantiate();
        let ticket_id = raffle.new_ticket(None);

        assert!(raffle.count == 1 && raffle.tickets.len() == 1 && ticket_id == 1)
    }

    #[test]
    fn Raffle_new_ticket_with_lines() {
        let mut raffle = Raffle::instantiate();
        let len80 = 80;
        let ticket_id = raffle.new_ticket(Some(len80));

        assert!(
            raffle.count == 1
                && raffle.tickets.len() == 1
                && ticket_id == *raffle.tickets.keys().nth(0).unwrap()
                && len80 as usize == raffle.tickets.get(&ticket_id).unwrap().line_list.len()
        )
    }

    #[test]
    fn Raffle_get_ticket_success() {
        let mut raffle = Raffle::instantiate();
        raffle.new_ticket(None);
        let existing_id = 1;

        assert!(raffle.get_ticket(existing_id).is_ok())
    }

    #[test]
    fn Raffle_get_ticket_fail() {
        let mut raffle = Raffle::instantiate();
        raffle.new_ticket(None);
        let nonexistent_id = 100;

        assert!(raffle.get_ticket(nonexistent_id).is_err())
    }

    #[test]
    fn Raffle_append_ticket_success() {
        let mut raffle = Raffle::instantiate();
        raffle.new_ticket(None);
        let existing_id = 1;

        assert!(raffle.append_ticket(existing_id, 10).is_ok())
    }

    #[test]
    fn Raffle_append_ticket_fail() {
        let mut raffle = Raffle::instantiate();
        let nonexistent_id = 100;

        assert!(raffle.append_ticket(nonexistent_id, 10).is_err())
    }

    #[test]
    fn Raffle_evaluate_ticket_success() {
        let mut raffle = Raffle::instantiate();
        let ticket = static_ticket!();
        assert!(raffle.tickets.insert(1, ticket).is_none());

        assert!(raffle.evaluate_ticket(1).is_ok())
    }

    #[test]
    fn Raffle_evaluate_ticket_failure() {
        let mut raffle = Raffle::instantiate();
        let ticket = static_ticket!();
        assert!(raffle.tickets.insert(1, ticket).is_none());

        assert!(raffle.evaluate_ticket(2).is_err())
    }

    #[test]
    fn Raffle_get_ticket_list_type() {
        let raffle = Raffle::instantiate();
        assert!(raffle.get_ticket_list().is_array())
    }

    #[test]
    fn Raffle_get_ticket_list_composition() {
        let mut raffle = Raffle::instantiate();
        let ticket = static_ticket!();
        let idx = 1;
        assert!(raffle.tickets.insert(idx, ticket).is_none());
        let output: Vec<u8> = LINE_EVAL_VALUES.iter().map(|i| *i).collect();

        assert_eq!(
            raffle.get_ticket_list(),
            json!([{"id": idx, "lines": output}])
        );
    }

    // Test data
    static LINE_SEED_VALUES: [(u8, u8, u8); 50] = [
        (1, 0, 2),
        (2, 2, 2),
        (0, 0, 0),
        (1, 1, 2),
        (0, 1, 0),
        (0, 2, 2),
        (0, 2, 1),
        (2, 2, 2),
        (1, 1, 1),
        (2, 2, 2),
        (0, 2, 1),
        (2, 1, 1),
        (1, 0, 2),
        (2, 2, 0),
        (1, 2, 0),
        (0, 0, 2),
        (1, 1, 0),
        (1, 1, 1),
        (1, 0, 1),
        (2, 0, 1),
        (0, 0, 1),
        (2, 2, 0),
        (0, 0, 1),
        (0, 0, 0),
        (2, 0, 0),
        (0, 2, 0),
        (1, 0, 1),
        (1, 1, 2),
        (2, 1, 1),
        (1, 2, 1),
        (0, 2, 1),
        (0, 1, 1),
        (0, 2, 0),
        (0, 2, 0),
        (1, 2, 2),
        (1, 0, 2),
        (0, 0, 0),
        (2, 1, 1),
        (1, 1, 0),
        (1, 1, 1),
        (1, 2, 1),
        (2, 1, 1),
        (2, 1, 2),
        (0, 1, 1),
        (0, 0, 2),
        (1, 2, 2),
        (2, 0, 2),
        (0, 0, 0),
        (0, 2, 0),
        (0, 2, 0),
    ];
    static LINE_EVAL_VALUES: [u8; 50] = [
        1, 5, 5, 0, 0, 1, 1, 5, 5, 5, 1, 1, 1, 0, 1, 10, 10, 5, 10, 1, 0, 0, 0, 5, 10, 10, 10, 0,
        1, 0, 1, 10, 10, 10, 1, 1, 5, 1, 10, 5, 0, 1, 0, 10, 10, 1, 0, 5, 10, 10,
    ];
}
