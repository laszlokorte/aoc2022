#[derive(Clone, Copy)]
enum Move {
    Rock,
    Paper,
    Scisor,
}

#[derive(Clone, Copy)]
enum GameResult {
    RightWin,
    LeftWin,
    Draw,
}

impl Move {
    fn play(self, right: Self) -> GameResult {
        match (self, right) {
            (Move::Rock, Move::Rock) => GameResult::Draw,
            (Move::Paper, Move::Paper) => GameResult::Draw,
            (Move::Scisor, Move::Scisor) => GameResult::Draw,
            (Move::Rock, Move::Scisor) => GameResult::LeftWin,
            (Move::Paper, Move::Rock) => GameResult::LeftWin,
            (Move::Scisor, Move::Paper) => GameResult::LeftWin,
            (Move::Rock, Move::Paper) => GameResult::RightWin,
            (Move::Paper, Move::Scisor) => GameResult::RightWin,
            (Move::Scisor, Move::Rock) => GameResult::RightWin,
        }
    }

    fn play_for_goal(self, goal: GameResult) -> Self {
        match (self, goal) {
            (Move::Rock, GameResult::LeftWin) => Move::Scisor,
            (Move::Paper, GameResult::LeftWin) => Move::Rock,
            (Move::Scisor, GameResult::LeftWin) => Move::Paper,
            (Move::Rock, GameResult::RightWin) => Move::Paper,
            (Move::Paper, GameResult::RightWin) => Move::Scisor,
            (Move::Scisor, GameResult::RightWin) => Move::Rock,
            (Move::Rock, GameResult::Draw) => Move::Rock,
            (Move::Paper, GameResult::Draw) => Move::Paper,
            (Move::Scisor, GameResult::Draw) => Move::Scisor,
        }
    }
}

pub fn process_move(text: String) -> u32 {
    let lines = text.split("\n");

    return lines
        .map(|line| {
            let (left, right) = line.split_once(" ").unwrap();

            let left_move = match left.chars().next() {
                Some('A') => Move::Rock,
                Some('B') => Move::Paper,
                Some('C') => Move::Scisor,
                _ => panic!("unknown move {}", left),
            };

            let right_move = match right.chars().next() {
                Some('X') => Move::Rock,
                Some('Y') => Move::Paper,
                Some('Z') => Move::Scisor,
                _ => panic!("unknown move {}", left),
            };

            let move_score = match right_move {
                Move::Rock => 1,
                Move::Paper => 2,
                Move::Scisor => 3,
            };

            let result = left_move.play(right_move);

            let game_score = match result {
                GameResult::RightWin => 6,
                GameResult::LeftWin => 0,
                GameResult::Draw => 3,
            };

            return move_score + game_score;
        })
        .sum();
}

pub fn process_goal(text: String) -> u32 {
    let lines = text.split("\n");

    return lines
        .map(|line| {
            let (left, right) = line.split_once(" ").unwrap();

            let left_move = match left.chars().next() {
                Some('A') => Move::Rock,
                Some('B') => Move::Paper,
                Some('C') => Move::Scisor,
                _ => panic!("unknown move {}", left),
            };

            let right_goal = match right.chars().next() {
                Some('X') => GameResult::LeftWin,
                Some('Y') => GameResult::Draw,
                Some('Z') => GameResult::RightWin,
                _ => panic!("unknown move {}", left),
            };

            let right_move = left_move.play_for_goal(right_goal);

            let move_score = match right_move {
                Move::Rock => 1,
                Move::Paper => 2,
                Move::Scisor => 3,
            };

            let game_score = match right_goal {
                GameResult::RightWin => 6,
                GameResult::LeftWin => 0,
                GameResult::Draw => 3,
            };

            return move_score + game_score;
        })
        .sum();
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
        assert_eq!(process_move(MOVES.to_string()), 15);
    }

    #[test]
    fn test_process_goal() {
        const MOVES: &str = "\
        A Y\n\
        B X\n\
        C Z\
        ";
        assert_eq!(process_goal(MOVES.to_string()), 12);
    }
}
