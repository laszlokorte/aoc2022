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
        return Ok(m);
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
type ClosedValves = BTreeSet<usize>;
#[derive(Debug, Eq, PartialEq, Clone)]
struct State<const PLAYERS: usize> {
    turn: usize,
    just_moved: [bool; PLAYERS],
    time: u32,
    pressure_released: u32,
    open_flow: u32,
    closed: ClosedValves,
    current_position: [usize; PLAYERS],
}
pub fn process(input: String) -> Option<u32> {
    let (_, conns) = connections(&input).ok()?;
    let mut graph = Graph::try_from(&conns).ok()?;
    graph.floyd_warshall();
    let reduced_graph = graph.without_nodes(
        &conns
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if n.flow_rate > 0 || n.name == "AA" {
                    None
                } else {
                    Some(i)
                }
            })
            .collect(),
    );
    let start_pos = reduced_graph.labels.iter().position(|l| l == &"AA")?;
    let best = optimize_graph::<1>(&reduced_graph, start_pos, 30);
    best.map(|s| s.pressure_released)
}

fn optimize_graph<const PLAYERS: usize>(
    graph: &Graph,
    start_position: usize,
    time_limit: u32,
) -> Option<State<PLAYERS>> {
    let initial_state = State {
        turn: 0,
        just_moved: [false; PLAYERS],
        time: 0,
        open_flow: 0,
        pressure_released: 0,
        closed: (0..graph.labels.len())
            .filter(|n| graph.nodes[*n] > 0)
            .into_iter()
            .collect(),
        current_position: [start_position; PLAYERS],
    };

    let mut queue = std::collections::VecDeque::new();
    queue.push_back(initial_state.clone());
    let mut best = initial_state.clone();

    while let Some(State {
        turn,
        just_moved,
        open_flow,
        time,
        pressure_released,
        current_position,
        closed,
    }) = queue.pop_front()
    {
        if time >= time_limit {
            if pressure_released > best.pressure_released {
                best = State {
                    turn,
                    just_moved,
                    open_flow,
                    time,
                    pressure_released,
                    current_position,
                    closed,
                };
            }
            continue;
        }
        if closed.is_empty() {
            queue.push_back(State {
                turn,
                just_moved: [false; PLAYERS],
                open_flow,
                closed: closed.clone(),
                current_position,
                pressure_released: pressure_released + (time_limit - time) * open_flow,
                time: time_limit,
            });
        }
        let can_move = &graph.matrix[current_position[turn]];

        if closed.contains(&current_position[turn]) {
            let mut opened = closed.clone();
            opened.remove(&current_position[turn]);
            let reduction = graph.nodes[current_position[turn]];
            queue.push_back(State {
                turn,
                just_moved: [false; PLAYERS],
                open_flow: open_flow + reduction,
                closed: opened,
                current_position,
                pressure_released: pressure_released + open_flow,
                time: time + 1,
            })
        }
        if just_moved[turn] {
            continue;
        }
        for (target, time_needed) in can_move.iter().enumerate() {
            if let Some(t) = time_needed {
                if time + t <= time_limit {
                    queue.push_back(State {
                        turn,
                        just_moved: [true; PLAYERS],
                        open_flow,
                        closed: closed.clone(),
                        current_position: [target; PLAYERS],
                        pressure_released: pressure_released + open_flow * t,
                        time: time + t,
                    })
                }
            }
        }
    }

    Some(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II";

        assert_eq!(process(COMMANDS.to_string()), Some(1651));
    }
}
