#![feature(extern_types)]
use std::{collections::{HashSet, BinaryHeap, HashMap}, cmp::Ordering};

use js_sys::{Number, Reflect};
use leptos::*;
use leptos::ev::KeyboardEvent;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
mod rand;
mod utils;
mod peer;
mod board;
use board::{Board, RobotPositions, Direction};
mod net;

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

#[derive(Clone, PartialEq, Eq)]
pub struct Bid {
    timestamp: u32,
    bid: u32,
    name: String,
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(if self.bid < other.bid { 
            Ordering::Less
        } else if self.bid > other.bid {
            Ordering::Greater
        } else if self.timestamp < other.timestamp {
            Ordering::Less
        } else if self.timestamp > other.timestamp {
            Ordering::Greater
        } else {
            Ordering::Equal
        })
    }
}
impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Default)]
pub struct RoomState {
    players: HashMap<String, String>,
    bids: BinaryHeap<Bid>
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
        let room_state: RwSignal<RoomState> = create_rw_signal(cx, Default::default());
        let network_state = create_rw_signal(cx, NetworkState::None);
        let (board, set_board) = create_signal(cx, Board::generate(16, 16));
        let positions = create_rw_signal(cx, board().initial_positions);
        let moves = create_rw_signal(cx, Vec::new());

        // clear room state when network state is set to None
        create_effect(cx, move |_| {
            let state = network_state.get();
            match state {
                NetworkState::None => { room_state.set(Default::default()); }
                _ => {}
            }
        });

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
                        let md: JsValue = conn.metadata();
                        let name = match Reflect::get(&md, &JsValue::from_str("name")).and_then(|v| v.as_string().ok_or(JsValue::NULL)) {
                            Ok(s) => s,
                            Err(_) => "Anonymous".to_string()
                        };

                        network_state.update(|state| {
                            if let NetworkState::Server { conns, .. } = state {
                                // Broadcast the PlayerJoin message
                                peer::broadcast(&conns,
                                    &net::Message::PlayerJoin(net::PlayerJoinMessage {
                                        ids: vec![conn.peer()],
                                        names: vec![name.clone()]
                                    })
                                );

                                // Update network state & room state to include new player
                                conns.push(conn.clone());
                                room_state.update(|room| {
                                    room.players.insert(conn.peer(), name);
                                });
                            }
                        });

                        // Register event handlers for the connection
                        let conn_clone = conn.clone();
                        conn.on("open", &Closure::<dyn Fn()>::new(move || {
                            peer::send(&conn_clone, 
                                &net::Message::BoardState(net::BoardStateMessage {
                                    board: board.get(),
                                })
                            );
                        }).into_js_value());

                        // Broadcast `PlayerLeave` message when
                        // the player disconnects.
                        let id =conn.peer();
                        conn.on("close", &Closure::<dyn Fn()>::new(move || {
                            let id = id.clone();
                            network_state.update(move |state| {
                                if let NetworkState::Server { conns, .. } = state {
                                    peer::broadcast(&conns, 
                                        &net::Message::PlayerLeave(net::PlayerLeaveMessage {
                                            id: id,
                                        })
                                    );                                    
                                };
                            })
                        }).into_js_value());
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