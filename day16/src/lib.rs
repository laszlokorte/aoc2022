#![feature(iter_array_chunks)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::separated_list0;
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::*;
use std::collections::BTreeSet;
use std::collections::HashMap;

#[derive(Debug)]
struct Node<'a> {
    name: &'a str,
    flow_rate: u32,
    connections: Vec<&'a str>,
}

fn connections(input: &str) -> IResult<&str, Vec<Node>> {
    separated_list0(
        line_ending,
        pair(
            separated_pair(
                preceded(tag("Valve "), character::complete::alpha1),
                tag(" has flow rate="),
                character::complete::u32,
            ),
            alt((
                preceded(
                    tag("; tunnels lead to valves "),
                    separated_list1(tag(", "), character::complete::alpha1),
                ),
                preceded(
                    tag("; tunnel leads to valve "),
                    character::complete::alpha1.map(|c| vec![c]),
                ),
            )),
        )
        .map(|((name, flow_rate), connections)| Node {
            name,
            flow_rate,
            connections,
        }),
    )(input)
}

#[derive(Debug)]
struct Graph<'s> {
    matrix: Vec<Vec<Option<u32>>>,
    labels: Vec<&'s str>,
    nodes: Vec<u32>,
}

impl<'a> TryFrom<&Vec<Node<'a>>> for Graph<'a> {
    type Error = String;
    fn try_from(value: &Vec<Node<'a>>) -> Result<Self, String> {
        let mut m = Self {
            labels: value.iter().map(|n| n.name).collect(),
            nodes: value.iter().map(|n| n.flow_rate).collect(),
            matrix: vec![vec![None; value.len()]; value.len()],
        };

        for r in 0..value.len() {
            m.matrix[r][r] = Some(0);
        }

        for (s, con) in value.iter().enumerate() {
            for target_name in &con.connections {
                let t = value
                    .iter()
                    .position(|n| &n.name == target_name)
                    .ok_or("unknown node")?;
                m.matrix[s][t] = Some(1);
            }
        }
        Ok(m)
    }
}
impl Graph<'_> {
    fn floyd_warshall(&mut self) {
        let node_count = self.matrix.len();

        for k in 0..node_count {
            for i in 0..node_count {
                for j in 0..node_count {
                    let sum = self.matrix[i][k].and_then(|a| Some(a + self.matrix[k][j]?));
                    if let Some(min) = sum {
                        self.matrix[i][j] =
                            Some(std::cmp::min(self.matrix[i][j].unwrap_or(min), min));
                    }
                }
            }
        }
    }

    fn without_nodes(&self, to_remove: &BTreeSet<usize>) -> Self {
        Self {
            labels: self
                .labels
                .iter()
                .enumerate()
                .filter_map(|(i, l)| {
                    if to_remove.contains(&i) {
                        None
                    } else {
                        Some(*l)
                    }
                })
                .collect(),
            nodes: self
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(i, l)| {
                    if to_remove.contains(&i) {
                        None
                    } else {
                        Some(*l)
                    }
                })
                .collect(),
            matrix: self
                .matrix
                .iter()
                .enumerate()
                .filter_map(|(c, col)| {
                    if !to_remove.contains(&c) {
                        Some(
                            col.iter()
                                .enumerate()
                                .filter_map(|(r, weight)| {
                                    if !to_remove.contains(&r) {
                                        Some(weight)
                                    } else {
                                        None
                                    }
                                })
                                .copied()
                                .collect::<Vec<Option<u32>>>(),
                        )
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

impl std::fmt::Display for Graph<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.matrix
                .iter()
                .map(|r| {
                    r.iter()
                        .map(|r| match r {
                            None => ".,".to_owned(),
                            Some(s) => format!("{s},"),
                        })
                        .collect::<String>()
                })
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Default)]
struct State {
    time_left: u32,
    current_position: usize,
    open: usize,
}

impl State {
    fn visit(&self, position: usize, duration: u32) -> Option<State> {
        let mask = 1 << position;
        let Some(time_left) = self.time_left.checked_sub(duration + 1) else {
            return None
        };

        if mask & self.open > 0 {
            return None
        }

        Some(Self {
            time_left,
            current_position: position,
            open: self.open | mask
        })
    }
}

#[derive(Debug,Default)]
struct Optimizer {
    cache: HashMap<State, u32>
}

impl Optimizer{

    fn best(&mut self, graph: &Graph, state: &State) -> Option<u32> {
        if let Some(result) = self.cache.get(state) {
            return Some(*result)
        }

        let mut max = 0;
        let Some(neighbours) = graph.matrix.get(state.current_position) else {
            return None
        };

        for (neighbour, &distance) in neighbours.iter().enumerate() {
            let Some(d) = distance else {
                continue
            };

            let Some(new_state) = state.visit(neighbour, d) else {
                continue
            };

            let Some(flow) = graph.nodes.get(neighbour) else {
                continue
            };

            let Some(b) = self.best(graph, &new_state) else {
                continue;
            };

            max = u32::max(max, b + flow * new_state.time_left);
        }
        
        Some(max)
    }
}

pub fn process_single(
    input: String,
    start_pos_label: &str,
    time_limit: u32,
) -> Option<u32> {
    let (_, conns) = connections(&input).ok()?;
    let mut graph = Graph::try_from(&conns).ok()?;
    graph.floyd_warshall();
    let reduced_graph = graph.without_nodes(
        &conns
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if n.flow_rate > 0 || n.name == start_pos_label {
                    None
                } else {
                    Some(i)
                }
            })
            .collect(),
    );
    let start_pos = reduced_graph
        .labels
        .iter()
        .position(|l| l == &start_pos_label)?;
    let mut optimizer = Optimizer::default();
    let initial_state = State {
        time_left: time_limit,
        current_position: start_pos,
        open: 0,
    };

    optimizer.best(&reduced_graph, &initial_state)
}


pub fn process_double(
    input: String,
    start_pos_label: &str,
    time_limit: u32,
) -> Option<u32> {
    let (_, conns) = connections(&input).ok()?;
    let mut graph = Graph::try_from(&conns).ok()?;
    graph.floyd_warshall();
    let reduced_graph = graph.without_nodes(
        &conns
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if n.flow_rate > 0 || n.name == start_pos_label {
                    None
                } else {
                    Some(i)
                }
            })
            .collect(),
    );
    let start_pos = reduced_graph
        .labels
        .iter()
        .position(|l| l == &start_pos_label)?;

    let mut optimizer = Optimizer::default();

    let partitions = ((1 << graph.nodes.len()) - 1) / 2;
    let mut best_sum = 0;

    for initial_mask in 0..=partitions {
        let initial_state_a = State {
            time_left: time_limit,
            current_position: start_pos,
            open: initial_mask,
        };

        let initial_state_b = State {
            time_left: time_limit,
            current_position: start_pos,
            open: !initial_mask,
        };

        let Some(best_a) = optimizer.best(&reduced_graph, &initial_state_a) else {
            continue;
        };

        let Some(best_b) = optimizer.best(&reduced_graph, &initial_state_b) else {
            continue;
        };

        best_sum = best_sum.max(best_a + best_b);
    }

    Some(best_sum)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");
        
        assert_eq!(
            process_single(include_str!("../input.txt").to_string(), "AA", 30),
            Some(1754)
        );
        assert_eq!(process_single(COMMANDS.to_string(), "AA", 30), Some(1651));
        assert_eq!(process_double(COMMANDS.to_string(), "AA", 26), Some(1707));
    }
}
