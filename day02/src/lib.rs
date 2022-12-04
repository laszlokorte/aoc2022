use std::str::FromStr;

#[derive(Clone, Copy)]
enum Move {
    Rock,
    Paper,
    Scisor,
}

#[derive(Clone, Copy)]
enum Outcome {
    RightWin,
    LeftWin,
    Draw,
}

impl Move {
    fn play(self, right: Self) -> Outcome {
        match (self, right) {
            (Move::Rock, Move::Rock) => Outcome::Draw,
            (Move::Paper, Move::Paper) => Outcome::Draw,
            (Move::Scisor, Move::Scisor) => Outcome::Draw,
            (Move::Rock, Move::Scisor) => Outcome::LeftWin,
            (Move::Paper, Move::Rock) => Outcome::LeftWin,
            (Move::Scisor, Move::Paper) => Outcome::LeftWin,
            (Move::Rock, Move::Paper) => Outcome::RightWin,
            (Move::Paper, Move::Scisor) => Outcome::RightWin,
            (Move::Scisor, Move::Rock) => Outcome::RightWin,
        }
    }

    fn play_for_goal(self, goal: Outcome) -> Self {
        match (self, goal) {
            (Move::Rock, Outcome::LeftWin) => Move::Scisor,
            (Move::Paper, Outcome::LeftWin) => Move::Rock,
            (Move::Scisor, Outcome::LeftWin) => Move::Paper,
            (Move::Rock, Outcome::RightWin) => Move::Paper,
            (Move::Paper, Outcome::RightWin) => Move::Scisor,
            (Move::Scisor, Outcome::RightWin) => Move::Rock,
            (Move::Rock, Outcome::Draw) => Move::Rock,
            (Move::Paper, Outcome::Draw) => Move::Paper,
            (Move::Scisor, Outcome::Draw) => Move::Scisor,
        }
    }
}

impl FromStr for Move {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('A') => Ok(Move::Rock),
            Some('B') => Ok(Move::Paper),
            Some('C') => Ok(Move::Scisor),
            Some('X') => Ok(Move::Rock),
            Some('Y') => Ok(Move::Paper),
            Some('Z') => Ok(Move::Scisor),
            _ => Err("Unknown Move".to_string()),
        }
    }
}

impl FromStr for Outcome {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('X') => Ok(Outcome::LeftWin),
            Some('Y') => Ok(Outcome::Draw),
            Some('Z') => Ok(Outcome::RightWin),
            _ => Err("Unknown Outcome".to_string()),
        }
    }
}

fn score_move(m: Move) -> u32 {
    match m {
        Move::Rock => 1,
        Move::Paper => 2,
        Move::Scisor => 3,
    }
}

fn score_outcome(o: Outcome) -> u32 {
    match o {
        Outcome::RightWin => 6,
        Outcome::LeftWin => 0,
        Outcome::Draw => 3,
    }
}

pub fn process_move(text: String) -> Result<u32, String> {
    let lines = text.split('\n');

    lines
        .map(|line| {
            let (left, right) = line.split_once(' ').unwrap();

            let left_move = Move::from_str(left);
            let right_move = Move::from_str(right);

            match (left_move, right_move) {
                (Ok(l), Ok(r)) => Ok(score_move(r) + score_outcome(l.play(r))),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            }
        })
        .fold(Ok(0), |acc, v| match (&acc, v) {
            (Ok(a), Ok(n)) => Ok(a + n),
            _ => acc,
        })
}

pub fn process_goal(text: String) -> Result<u32, String> {
    let lines = text.split('\n');

    lines
        .map(|line| {
            let (left, right) = line.split_once(' ').unwrap();

            let left_move = Move::from_str(left);
            let right_move = Outcome::from_str(right);

            match (left_move, right_move) {
                (Ok(l), Ok(r)) => Ok(score_outcome(r) + score_move(l.play_for_goal(r))),
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            }
        })
        .fold(Ok(0), |acc, v| match (&acc, v) {
            (Ok(a), Ok(n)) => Ok(a + n),
            _ => acc,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_move() {
        const MOVES: &str = "\
        A Y\n\
        B X\n\
        C Z\
        ";
        assert_eq!(process_move(MOVES.to_string()), Ok(15));
    }

    #[test]
    fn test_process_goal() {
        const MOVES: &str = "\
        A Y\n\
        B X\n\
        C Z\
        ";
        assert_eq!(process_goal(MOVES.to_string()), Ok(12));
    }
}
