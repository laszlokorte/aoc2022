#![feature(iter_array_chunks)]
#![feature(linked_list_cursors)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use std::collections::{HashMap, HashSet, VecDeque};

use nom::{
    branch::alt, bytes::complete::tag, character::complete::line_ending, combinator::value,
    multi::separated_list0, sequence::separated_pair, *,
};
#[derive(Debug, Clone)]
enum Expression<'a> {
    Constant(i64),
    BinaryOp(BinaryOperation, &'a str, &'a str),
}
enum EvalMissing {
    Both,
    Left,
    Right,
}
impl<'a> Expression<'a> {
    fn eval(&self, bindings: &'a HashMap<&str, i64>) -> Result<i64, EvalMissing> {
        match self {
            Expression::Constant(v) => Ok(*v),
            Expression::BinaryOp(op, a, b) => match (bindings.get(a), bindings.get(b)) {
                (Some(va), Some(vb)) => Ok(op.eval(*va, *vb)),
                (Some(..), None) => Err(EvalMissing::Right),
                (None, Some(..)) => Err(EvalMissing::Left),
                (None, None) => Err(EvalMissing::Both),
            },
        }
    }
}

#[derive(Clone, Debug)]
enum BinaryOperation {
    Addition,
    Subtraction,
    Division,
    Multiplication,
}

impl BinaryOperation {
    fn inverse_left<'b>(&self, x: &'b str, r: &'b str) -> (Self, &'b str, &'b str) {
        match self {
            // x = l + r => l = x - r
            Self::Addition => (Self::Subtraction, x, r),
            // x = l - r => l = x + r
            Self::Subtraction => (Self::Addition, x, r),
            // x = l / r => l = x * r
            Self::Division => (Self::Multiplication, x, r),
            // x = l * r => l = x / r
            Self::Multiplication => (Self::Division, x, r),
        }
    }
    fn inverse_right<'b>(&self, x: &'b str, l: &'b str) -> (Self, &'b str, &'b str) {
        match self {
            // x = l + r => r = x - l
            Self::Addition => (Self::Subtraction, x, l),
            // x = l - r => r = l - x
            Self::Subtraction => (Self::Subtraction, l, x),
            // x = l / r => r = l / x
            Self::Division => (Self::Multiplication, l, x),
            // x = l * r => r = x / l
            Self::Multiplication => (Self::Division, x, l),
        }
    }
}

impl BinaryOperation {
    fn eval(&self, a: i64, b: i64) -> i64 {
        match self {
            BinaryOperation::Addition => a + b,
            BinaryOperation::Subtraction => a - b,
            BinaryOperation::Multiplication => a * b,
            BinaryOperation::Division => a / b,
        }
    }
}

fn binary_expression(input: &str) -> IResult<&str, Expression> {
    let (input, op1) = character::complete::alpha1(input)?;
    let (input, operator) = alt((
        value(BinaryOperation::Addition, tag(" + ")),
        value(BinaryOperation::Subtraction, tag(" - ")),
        value(BinaryOperation::Multiplication, tag(" * ")),
        value(BinaryOperation::Division, tag(" / ")),
    ))(input)?;
    let (input, op2) = character::complete::alpha1(input)?;

    Ok((input, Expression::BinaryOp(operator, op1, op2)))
}
fn expression(input: &str) -> IResult<&str, Expression> {
    alt((
        binary_expression,
        character::complete::i64.map(Expression::Constant),
    ))(input)
}
fn node(input: &str) -> IResult<&str, (&str, Expression)> {
    separated_pair(character::complete::alpha1, tag(": "), expression)(input)
}

fn solve<'s, 'a>(
    map: &'a HashMap<&'s str, Expression<'s>>,
    root: &'s str,
    evaluated: &mut HashMap<&'s str, i64>,
    reversed_map: &'a mut HashMap<&'s str, Expression<'s>>,
) -> Option<i64> {
    let mut visited = HashSet::<&str>::new();
    let mut seen = HashSet::<&str>::new();
    let mut stack = VecDeque::<&str>::new();

    stack.push_front(root);
    seen.insert(root);
    while let Some(current) = stack.pop_front() {
        let Some(expr) = map.get(&current) else {
            continue;
        };
        if !visited.insert(current) {
            match expr.eval(evaluated) {
                Ok(value) => {
                    evaluated.insert(current, value);
                    reversed_map.insert(current, expr.clone());
                }
                Err(e) => {
                    let Expression::BinaryOp(op, left, right) = expr else {
                        return None;
                    };
                    match e {
                        EvalMissing::Left => {
                            let (op, l, r) = op.inverse_left(current, right);
                            let new_expr = Expression::BinaryOp(op, l, r);
                            reversed_map.insert(left, new_expr);
                        }
                        EvalMissing::Right => {
                            let (op, l, r) = op.inverse_right(current, left);
                            reversed_map.insert(right, Expression::BinaryOp(op, l, r));
                        }
                        EvalMissing::Both => {}
                    }
                }
            }
        } else {
            stack.push_front(current);
            if let Expression::BinaryOp(_, a, b) = expr {
                if seen.insert(a) {
                    stack.push_front(a);
                }
                if seen.insert(b) {
                    stack.push_front(b);
                }
            }
        }
    }
    evaluated.get(root).cloned()
}
pub fn process(input: String) -> Option<i64> {
    let (_, nodes) = separated_list0(line_ending, node)(&input).ok()?;
    let map: HashMap<&str, Expression> = nodes.into_iter().collect();
    let mut evaluated = HashMap::<&str, i64>::new();
    let mut _reversed = HashMap::new();

    solve(&map, "root", &mut evaluated, &mut _reversed)
}

pub fn process_solve(input: String) -> Option<i64> {
    let (_, nodes) = separated_list0(line_ending, node)(&input).ok()?;
    let mut map: HashMap<&str, Expression> = nodes.into_iter().collect();
    let mut reversed = HashMap::new();
    map.remove("humn")?;
    let root = map.remove("root")?;
    let Expression::BinaryOp(_, left, right) = root else {
        return None;
    };
    let mut evaluated = HashMap::<&str, i64>::new();
    let left_solution = solve(&map, left, &mut evaluated, &mut reversed);
    let right_solution = solve(&map, right, &mut evaluated, &mut reversed);
    let root_target_value = left_solution.or(right_solution)?;
    evaluated.insert(left, root_target_value);
    evaluated.insert(right, root_target_value);
    solve(&reversed, "humn", &mut evaluated, &mut HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(process(COMMANDS.to_string()), Some(152));
        assert_eq!(process_solve(COMMANDS.to_string()), Some(301));
    }
}
