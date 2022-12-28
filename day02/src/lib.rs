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

pub fn process_move(text: String) -> Option<u32> {
    let lines = text.split('\n');

    lines
        .map(|line| {
            let (left, right) = line.split_once(' ')?;

            let left_move = Move::from_str(left).ok()?;
            let right_move = Move::from_str(right).ok()?;

            Some(score_move(right_move) + score_outcome(left_move.play(right_move)))
        })
        .sum()
}

pub fn process_goal(text: String) -> Option<u32> {
    let lines = text.split('\n');

    lines
        .map(|line| {
            let (left, right) = line.split_once(' ')?;

            let left_move = Move::from_str(left).ok()?;
            let right_move = Outcome::from_str(right).ok()?;

            Some(score_outcome(right_move) + score_move(left_move.play_for_goal(right_move)))
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOVES: &str = include_str!("test.txt");

    #[test]
    fn test_process_move() {
        assert_eq!(process_move(MOVES.to_string()), Some(15));
    }

    #[test]
    fn test_process_goal() {
        assert_eq!(process_goal(MOVES.to_string()), Some(12));
    }
}
