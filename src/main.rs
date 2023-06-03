#![feature(extern_types)]
use std::collections::HashSet;

use js_sys::Number;
use leptos::*;
use leptos::ev::KeyboardEvent;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
mod rand;
mod utils;
mod peer;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}
impl Direction {
    pub fn id(self) -> &'static str {
        match self {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right"
        }
    }
}
type RobotPositions = [usize; 5];

#[derive(Clone)]
pub struct Board {
    pub width: usize,

    /// A width * (height - 1) size array.
    /// There is a wall between (i, j) and (i, j+1) iff
    /// horizontal_walls[i, j] is true.
    pub horizontal_walls: Vec<bool>,

    /// A (width - 1) * height size array.
    /// There is a wall between (i, j) and (i+1, j) iff
    /// verticall_walls[i, j] is true.
    pub vertical_walls: Vec<bool>,

    // Initial position of the robots
    pub initial_positions: RobotPositions
}

const RED: usize = 0;
const YELLOW: usize = 1;
const GREEN: usize = 2;
const BLUE: usize = 3;
const BLACK: usize = 4;

impl Board {
    pub fn height(&self) -> usize {
        self.horizontal_walls.len() / self.width + 1
    }

    pub fn generate(width: usize, height: usize) -> Self {
        let mut board = Board {
            width,
            horizontal_walls: vec![false; width * (height - 1)],
            vertical_walls: vec![false; (width - 1) * height],
            initial_positions: [0, 1, 2, 3, 4]
        };
        

        let mut used_tiles: HashSet<(usize, usize)> = HashSet::new();

        for _ in 0..16 {
            let mut i: usize = 0;
            let mut j: usize = 0;
            loop {
                i = rand::uniform(1, board.width - 1);
                j = rand::uniform(1, board.height() - 1);
                if false
                    || used_tiles.contains(&(i+1, j-1))
                    || used_tiles.contains(&(i+1, j))
                    || used_tiles.contains(&(i+1, j+1))
                    || used_tiles.contains(&(i, j-1))
                    || used_tiles.contains(&(i, j))
                    || used_tiles.contains(&(i, j+1))
                    || used_tiles.contains(&(i-1, j-1))
                    || used_tiles.contains(&(i-1, j))
                    || used_tiles.contains(&(i-1, j+1)) {
                    continue;
                }
                break;
            }
            used_tiles.insert((i, j));

            board.horizontal_walls[(j + rand::uniform(0, 2) - 1)*width+i] = true;
            board.vertical_walls[j*(width-1)+i+ rand::uniform(0, 2) - 1] = true;    
        }

        board.horizontal_walls[rand::uniform(4, 7)*width] = true;
        board.horizontal_walls[rand::uniform(9, 12)*width] = true;
        board.horizontal_walls[rand::uniform(4, 7)*width+(width-1)] = true;
        board.horizontal_walls[rand::uniform(9, 12)*width+(width-1)] = true;

        board.vertical_walls[rand::uniform(4, 7)] = true;
        board.vertical_walls[rand::uniform(9, 12)] = true;
        board.vertical_walls[rand::uniform(4, 7)+(height-1)*(width-1)] = true;
        board.vertical_walls[rand::uniform(9, 12)+(height-1)*(width-1)] = true;

        // block off center tiles
        board.vertical_walls[width/2 - 2 + (height/2-1)*(width - 1)] = true;
        board.vertical_walls[width/2 + 0 + (height/2-1)*(width - 1)] = true;
        board.vertical_walls[width/2 - 2 + (height/2)*(width - 1)] = true;
        board.vertical_walls[width/2 + 0 + (height/2)*(width - 1)] = true;
        board.horizontal_walls[width/2 - 1 + (height/2 - 2)*width] = true;
        board.horizontal_walls[width/2 + 0 + (height/2 - 2)*width] = true;
        board.horizontal_walls[width/2 - 1 + (height/2 + 0)*width] = true;
        board.horizontal_walls[width/2 + 0 + (height/2 + 0)*width] = true;

        board
    }

    /// Returns whether the given tile index
    /// represents a center (blocked-off) tile.
    pub fn is_center_tile(&self, tile: usize) -> bool {
        let x = tile % self.width;
        let y = tile / self.width;
        let height = self.height();

        self.width / 2 - 1 <= x && x <= self.width / 2 &&
            height / 2 - 1 <= y && y <= height / 2
    }

    /// Given a robot position, move a robot in
    /// a direction. Returns the new robot positions.
    pub fn move_robot(&self, mut positions: RobotPositions, robot: usize, direction: Direction) -> RobotPositions {
        let width = self.width;
        let height = self.height();
        match direction {
            Direction::Down => {
                let wall_pos = self.horizontal_walls.iter().enumerate()
                    .filter(|(i, b)| (**b) && (*i % width  == positions[robot] % width))
                    .map(|(i, _)| i / width)
                    .filter(|&i| i + 1 > positions[robot] / width)
                    .next().unwrap_or(height - 1);
                let robot_pos = positions.iter()
                    .filter(|&i| i % width == positions[robot] % width)
                    .map(|i| i / width)
                    .filter(|&i| i > positions[robot] / width)
                    .min().map(|i| i - 1).unwrap_or(height - 1);
                let target_pos = usize::min(wall_pos, robot_pos);

                positions[robot] = positions[robot] % width + target_pos * width;
                positions   
            },
            Direction::Up => {
                let wall_pos = self.horizontal_walls.iter().enumerate().rev()
                    .filter(|(i, b)| (**b) && (*i % width  == positions[robot] % width))
                    .map(|(i, _)| i / width)
                    .filter(|&i| i < positions[robot] / width)
                    .next().map(|i| i + 1).unwrap_or(0);
                let robot_pos = positions.iter()
                    .filter(|&i| i % width == positions[robot] % width)
                    .map(|i| i / width)
                    .filter(|&i| i < positions[robot] / width)
                    .max().map(|i| i + 1).unwrap_or(0);
                let target_pos = usize::max(wall_pos, robot_pos);

                positions[robot] = positions[robot] % width + target_pos * width;
                positions
            },
            Direction::Right => {
                let wall_pos = self.vertical_walls.iter().enumerate()
                    .filter(|&(i, b)| (*b && i / (width - 1) == positions[robot] / width))
                    .map(|(i, _)| i % (width - 1))
                    .filter(|&i| i + 1 > positions[robot] % width)
                    .next().map(|i| i).unwrap_or(width - 1);
                let robot_pos = positions.iter()
                    .filter(|&i| i / width == positions[robot] / width)
                    .map(|i| i % width)
                    .filter(|&i| i > positions[robot] % width)
                    .min().map(|i| i - 1).unwrap_or(width - 1);
                let target_pos = usize::min(wall_pos, robot_pos);

                positions[robot] = (positions[robot] / width) * width + target_pos;
                positions
            },
            Direction::Left => {
                let wall_pos = self.vertical_walls.iter().enumerate().rev()
                    .filter(|&(i, b)| *b && i / (width - 1) == positions[robot] / width)
                    .map(|(i, _)| i % (width - 1))
                    .filter(|&i| i < positions[robot] % width)
                    .next().map(|i| i + 1).unwrap_or(0);
                let robot_pos = positions.iter()
                    .filter(|&i| i / width == positions[robot] / width)
                    .map(|i| i % width)
                    .filter(|&i| i < positions[robot] % width)
                    .max().map(|i| i + 1).unwrap_or(0);
                let target_pos = usize::max(wall_pos, robot_pos);

                positions[robot] = (positions[robot] / width) * width + target_pos;
                positions
            }
        }
    }
}

#[component]
pub fn BoardWidget(cx: Scope, board: ReadSignal<Board>, positions: Option<RwSignal<RobotPositions>>, moves: RwSignal<Vec<(usize, Direction)>>) -> impl IntoView {
    // invariant: if set_position is None, moves is empty
    let (positions, set_positions) = match positions {
        None => (Signal::derive(cx, move || board().initial_positions), None),
        Some(rw) => (rw.into(), Some(rw.write_only()))
    };

    let keydown = move |robot, evt: KeyboardEvent| {
        let positions = positions();

        let dir = match evt.code().as_str() {
            "ArrowDown" => Direction::Down,
            "ArrowUp" => Direction::Up,
            "ArrowRight" => Direction::Right,
            "ArrowLeft" => Direction::Left,
            _ => {return}
        };

        let new_positions = board().move_robot(positions, robot, dir);
        if new_positions != positions {
            set_positions.unwrap().set(new_positions);
            moves.update(|moves| moves.push((robot, dir)));
        }
    };

    view !{
        cx, 
        <div class="board" style={format!("width:{}px;height:{}px", 32 * board.get().width, 32 * board.get().height())}>
            {move || if moves.get().len() != 0 {
                Some(view!{ cx, <div class="refresh" on:click={move |_| {
                    moves.update(|v| v.clear());
                    set_positions.unwrap().set(board().initial_positions);
                }}></div> })
            } else { None }}
            <For 
                each=move || 0..board.get().width*board.get().height()
                key=|&i| i
                view=move |cx, i| {
                    view! {
                        cx, 
                        <div class={format!("{} {}", "tile", if board.get().is_center_tile(i) { "center" } else { "" })} style={
                            let width = board.get().width;
                            format!("top:{}px;left:{}px", 32 * (i / width), 32 * (i % width))}></div>
                    }
                }/>

            <For
                each=move || 0..5
                key=|&i| i
                view=move |cx, i| {
                    view! {
                        cx,
                        <div class={move || format!("robot robot-{}", i)}
                            tabIndex="-1"
                            on:keydown={move |evt| if set_positions.is_some() { keydown(i, evt) }}
                            style={move || {
                                let width = board.get().width;
                                let pos = positions()[i];
                                format!("top:{}px;left:{}px", 32 * (pos/width), 32 * (pos%width))}
                        }></div>
                    }
                }
                />
            
            <For
                each={move || (board.get().horizontal_walls.iter().map(|x| *x).enumerate().collect::<Vec<_>>())}
                key=|(i, _)| *i
                view=move |cx, (i, exists)| {
                    view!{
                        cx, 
                        <Show
                            when=move || exists
                            fallback = |_|()>
                            <div class="wall-horizontal"
                                style={
                                    let width = board.get().width;
                                    format!("top:{}px;left:{}px", 32 * (i/width+1), 32 * (i%width))
                                }></div>
                        </Show>
                    }
                }/>

            <For
                each={move || (board.get().vertical_walls.iter().map(|x| *x).enumerate().collect::<Vec<_>>())}
                key=|(i, _)| *i
                view=move |cx, (i, exists)| {
                    view!{
                        cx, 
                        <Show
                            when=move || exists
                            fallback = |_|()>
                            <div class="wall-vertical"
                                style={
                                    let width = board.get().width;
                                    format!("top:{}px;left:{}px", 32 * (i/(width-1)), 32 * (i%(width-1)+1))
                                }></div>
                        </Show>
                    }
                }/>
        </div>
    }
}

#[component]
pub fn MoveList(cx: Scope, moves: ReadSignal<Vec<(usize, Direction)>>) -> impl IntoView {

    view! { cx,
        <div class="move-list">
            <For
                each={move || moves().into_iter().enumerate().collect::<Vec<_>>()}
                key={|&(i, _)| i}
                view=move |cx, (_, (robot, dir))| {
                    view! {cx, <span class={format!("move move-{} move-{}", robot, dir.id())}></span>}
                }
                />
        </div>
    }
}

#[derive(Clone)]
pub enum NetworkState {
    None,
    Client(peer::DataConnection),
    Server { peer: peer::Peer, conns: Vec<peer::DataConnection>, initialized: bool }
}

#[component]
pub fn Network(cx: Scope, state: RwSignal<NetworkState>) -> impl IntoView {
    // NOTE: Never directly set `state` to `None`

    let host = move |evt| {
        let id = format!("{:x}", rand::uniform(0, i32::MAX as usize));
        let peer = peer::Peer::new(&format!("ripoff-robots-{}", &id), object!{}.as_ref());
        state.set(NetworkState::Server { peer, conns: vec![], initialized: false });
    };

    let end_host = move |evt| {
        log!("destroying");
        if let NetworkState::Server { peer, .. } = state.get() {
            log!("destroying");
            peer.destroy();
        }
    };

    view! {
        cx,
        <div class="network-state">
            {move || match state.get() {
                NetworkState::None => {
                    view! {
                        cx, 
                        <div class="network-state-none">
                            <button on:click={host} class="network-button-host">"Host"</button>
                        </div>
                    }.into_any()
                },
                NetworkState::Server { peer, conns, .. } => {
                    view! {
                        cx,
                        <div class="network-state-host">
                            <div class="network-host-id">"ID: " {format!("{}", &peer.id()["ripoff-robots-".len()..])}</div>
                            <div>"Connections: "{conns.len()}</div>
                            <button on:click={end_host}>"End"</button>
                        </div>
                    }.into_any()
                },
                _ => unimplemented!()
            }}
        </div>
    }
}

pub fn main() {
    mount_to_body(|cx| {
        let network_state = create_rw_signal(cx, NetworkState::None);
        let (board, set_board) = create_signal(cx, Board::generate(16, 16));
        let positions = create_rw_signal(cx, board().initial_positions);
        let moves = create_rw_signal(cx, Vec::new());

        create_effect(cx, move |_| {
            let state = network_state.get();
            match state {
                NetworkState::None => {},
                NetworkState::Server { peer, conns, initialized: false } => {
                    peer.on("open", &Closure::<dyn Fn()>::new(move || {
                        log!("connection established to PeerServer")
                    }).into_js_value());

                    peer.on("close", &Closure::<dyn Fn()>::new(move || {
                        network_state.update(|state| {
                            *state = NetworkState::None;
                        })
                    }).into_js_value());


                    peer.on("connection", &Closure::<dyn Fn(_)>::new(move |conn: peer::DataConnection| {
                        network_state.update(|state| {
                            if let NetworkState::Server { conns, .. } = state {
                                conns.push(conn);
                            }
                        })
                    }).into_js_value());

                    network_state.update(|state| {
                        if let NetworkState::Server { initialized, .. } = state {
                            *initialized = true;
                        }
                    });
                },
                _ => {}
            }
        });

        view! { cx,  
            <Network state={network_state} />
            <BoardWidget board={board} positions={Some(positions)} moves={moves} />
            <MoveList moves={moves.read_only()} /> }
    })
}