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
    players: [PlayerState; PLAYERS],
    pressure_released: u32,
    closed: ClosedValves,
    visited: BTreeSet<usize>,
}

#[derive(Debug, Default, Copy, Eq, PartialEq, Clone)]
struct PlayerState {
    time: u32,
    just_moved: usize,
    current_position: usize,
    open_flow: u32,
}

pub fn process<const PLAYERS: usize>(
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
    let best = optimize_graph::<PLAYERS>(&reduced_graph, start_pos, time_limit);
    dbg!(&best);
    best.map(|s| s.pressure_released)
}
pub fn process_elephant(input: String) -> Option<u32> {
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
    let best = optimize_graph::<2>(&reduced_graph, start_pos, 26);
    dbg!(&best);
    best.map(|s| s.pressure_released)
}

fn optimize_graph<const PLAYERS: usize>(
    graph: &Graph,
    start_position: usize,
    time_limit: u32,
) -> Option<State<PLAYERS>> {
    let initial_state = State {
        visited: BTreeSet::default(),
        turn: 0,
        players: [PlayerState {
            current_position: start_position,
            ..PlayerState::default()
        }; PLAYERS],
        pressure_released: 0,
        closed: (0..graph.labels.len())
            .filter(|n| graph.nodes[*n] > 0)
            .into_iter()
            .collect(),
    };
    let mut queue = std::collections::VecDeque::new();
    queue.push_front(initial_state.clone());
    let mut best = initial_state.clone();
    while let Some(State {
        visited,
        turn,
        pressure_released,
        closed,
        players,
    }) = queue.pop_front()
    {
        let turn = turn % PLAYERS;
        let time = players[turn].time;
        // println!("{:?},{}", time, turn);
        if time >= time_limit {
            if turn + 1 < PLAYERS {
                queue.push_front(State {
                    visited: visited.clone(),
                    turn: turn + 1,
                    closed: closed.clone(),
                    pressure_released,
                    players,
                })
            } else if pressure_released > best.pressure_released {
                best = State {
                    visited: visited.clone(),
                    turn,
                    pressure_released,
                    players,
                    closed: closed.clone(),
                };
            }
            continue;
        }

        if closed.is_empty() && time < time_limit {
            let mut players_new = players;
            players_new[turn].just_moved = 0;
            players_new[turn].time += 1;
            queue.push_front(State {
                visited: visited.clone(),
                turn: turn + 1,
                closed: closed.clone(),
                pressure_released: pressure_released + players[turn].open_flow,
                players: players_new,
            });
        }
        let can_move = &graph.matrix[players[turn].current_position];

        if closed.contains(&players[turn].current_position) {
            let mut opened = closed.clone();
            opened.remove(&players[turn].current_position);
            let reduction = graph.nodes[players[turn].current_position];
            let mut players_new = players;
            let current_open_flow = players[turn].open_flow;
            players_new[turn].just_moved = 0;
            players_new[turn].time += 1;
            players_new[turn].open_flow += reduction;
            queue.push_front(State {
                visited: visited.clone(),
                turn: turn + 1,
                players: players_new,
                closed: opened,
                pressure_released: pressure_released + current_open_flow,
            })
        }
        if players[turn].just_moved > 0 {
            continue;
        }
        for (target, time_needed) in can_move.iter().enumerate() {
            if let Some(t) = time_needed {
                if time + t <= time_limit && t > &0 {
                    let mut players_new = players;
                    players_new[turn].just_moved += 1;
                    players_new[turn].current_position = target;
                    players_new[turn].time += *t;
                    let mut new_visited = visited.clone();
                    new_visited.insert(target);
                    queue.push_front(State {
                        visited: new_visited,
                        players: players_new,
                        turn: turn + 1,
                        closed: closed.clone(),
                        pressure_released: pressure_released + players[turn].open_flow * t,
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
        assert_eq!(
            process::<1>(include_str!("../input.txt").to_string(), "AA", 30),
            Some(1754)
        );
        assert_eq!(process::<1>(COMMANDS.to_string(), "AA", 30), Some(1651));
        // assert_eq!(process::<2>(COMMANDS.to_string(), "AA", 26), Some(1707));
    }
}
