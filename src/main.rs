#![feature(extract_if)]
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
use web_sys::console;
mod net;

#[component]
pub fn BoardWidget(cx: Scope, board: ReadSignal<Board>, positions: Option<RwSignal<RobotPositions>>, moves: RwSignal<Vec<(usize, Direction)>>) -> impl IntoView {
    // invariant: if set_position is None, moves is empty
    let (positions, set_positions) = match positions {
        None => (Signal::derive(cx, move || board.get().initial_positions), None),
        Some(rw) => (rw.into(), Some(rw.write_only()))
    };

    let keydown = move |robot, evt: KeyboardEvent| {
        let positions = positions.get();

        let dir = match evt.code().as_str() {
            "ArrowDown" => Direction::Down,
            "ArrowUp" => Direction::Up,
            "ArrowRight" => Direction::Right,
            "ArrowLeft" => Direction::Left,
            _ => {return}
        };

        let new_positions = board.get().move_robot(positions, robot, dir);
        if new_positions != positions {
            set_positions.unwrap().set(new_positions);
            moves.update(|moves| moves.push((robot, dir)));
        }
    };

    let tiles_memo = Signal::derive(cx, move || 0..board.get().width*board.get().height());
    let horizontal_memo = Signal::derive(cx, move || {
        let vec = board.get().horizontal_walls.iter().map(|x| *x).enumerate().filter(|&(i, b)| b).map(|(i, b)| i).collect::<Vec<_>>();
        log!("{}", vec.len());
        vec
    });
    let vertical_memo = Signal::derive(cx, move || 
        board.get().vertical_walls.iter().map(|x| *x).enumerate().filter(|&(i, b)| b).map(|(i, b)| i).collect::<Vec<_>>());

    view !{
        cx, 
        <div class="board" style={move || format!("width:{}px;height:{}px", 32 * board.get().width, 32 * board.get().height())}>
            {move || if moves.get().len() != 0 {
                Some(view!{ cx, <div class="refresh" on:click={move |_| {
                    moves.update(|v| v.clear());
                    set_positions.unwrap().set(board.get().initial_positions);
                }}></div> })
            } else { None }}
            <For 
                each=move || tiles_memo.get()
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
                                let pos = positions.get()[i];
                                format!("top:{}px;left:{}px", 32 * (pos/width), 32 * (pos%width))}
                        }></div>
                    }
                }
                />
            
            // 
            <For
                each=move || horizontal_memo.get()
                key=|&i| i
                view=move |cx, i| {
                    log!("rerendering horizontal walls");
                    view!{
                        cx, 
                        <div class="wall-horizontal"
                            style={
                                let width = board.get().width;
                                format!("top:{}px;left:{}px", 32 * (i/width+1), 32 * (i%width))
                            }></div>
                    }
                }/>

            <For
                each=move || vertical_memo.get()
                key=|&i| i
                view=move |cx, i| {
                    log!("rerendering vertical walls");
                    view!{
                        cx, 
                        <div class="wall-vertical"
                            style={
                                let width = board.get().width;
                                format!("top:{}px;left:{}px", 32 * (i/(width-1)), 32 * (i%(width-1)+1))
                            }></div>
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
                each={move || moves.get().into_iter().enumerate().collect::<Vec<_>>()}
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
    scores: HashMap<String, u32>,
    bids: BinaryHeap<Bid>
}

#[component]
pub fn Network(cx: Scope, state: RwSignal<NetworkState>, room_state: RwSignal<RoomState>, board: WriteSignal<Board>) -> impl IntoView {
    // NOTE: Never directly set `state` to `None`
    let room_id = create_rw_signal(cx, String::new());
    let name = create_rw_signal(cx, String::new());
    let join = move |evt| {
        log!("joining room {}", room_id.get());

        let id = format!("ripoff-robots-client-{:x}", rand::uniform(0, i32::MAX as usize));
        let peer = peer::Peer::new(&id, &JsValue::NULL);

        let peer_clone = peer.clone();
        peer.on("open", &Closure::<dyn Fn()>::new(move || {
            let options = object!{
                "metadata" => &object!{
                    "name" => name.get()
                }
            };
            let conn = peer_clone.connect(&format!("ripoff-robots-{}", room_id.get()), &options.into());
    
            state.set(NetworkState::Client (conn));
        }).into_js_value());
    };

    let host = move |evt| {
        let id = format!("{:x}", rand::uniform(0, i32::MAX as usize));
        let peer = peer::Peer::new(&format!("ripoff-robots-{}", &id), object!{}.as_ref());
        state.set(NetworkState::Server { peer, conns: vec![], initialized: false });
        room_state.update(|state| {
            state.players.insert("host".into(), name.get());
        });
    };

    let end_host = move |evt| {
        log!("destroying");
        if let NetworkState::Server { peer, .. } = state.get() {
            log!("destroying");
            peer.destroy();
        }
    };

    let end_client = move |evt| {
        if let NetworkState::Client(conn) = state.get() {
            conn.close();
        }
    };

    let randomize_board = move |evt| {
        board.set(Board::generate(16, 16));
    };

    view! {
        cx,
        <div class="network-state">
            {move || match state.get() {
                NetworkState::None => {
                    view! {
                        cx, 
                        <div class="network-state-none">
                            <input type="text" placeholder="name" prop:value={name}
                                on:input={move |ev| name.set(event_target_value(&ev))} />
                            <hr />
                            <button on:click={host} class="network-button-host">"Host"</button>
                            <hr />
                            <input type="text" placeholder="Room ID" 
                                prop:value={room_id}
                                on:input={move |ev| room_id.set(event_target_value(&ev))} />
                            <button on:click={join} class="network-button-join">"Join"</button>
                            <hr />
                            <button on:click={randomize_board}>"New Board"</button>
                        </div>
                    }.into_any()
                },
                NetworkState::Server { peer, conns, .. } => {
                    view! {
                        cx,
                        <div class="network-state-host">
                            <div class="network-host-id">"Room ID: " {format!("{}", &peer.id()["ripoff-robots-".len()..])}</div>
                            <div class="network-players">
                                <h3>"Players"</h3>
                                <For each={move || room_state.get().players.iter().map(|(id, name)| (id.to_owned(), name.to_owned())).collect::<Vec<_>>()}
                                    key=|(id,_name)| id.to_string()
                                    view=move |cx, (id, name)| {
                                        view!{
                                            cx, 
                                            <div class="network-player">
                                                <span class="network-player-name">{name}</span>
                                                <span class="network-player-score">{move || room_state.get().scores.get(&id).map(|x|*x).unwrap_or(0)}</span>
                                            </div>
                                        }
                                    }
                                    />
                            </div>
                            <button on:click={end_host}>"End"</button>
                        </div>
                    }.into_any()
                },
                NetworkState::Client(conn) => {
                    log!("rendering client");
                    view! {
                        cx,
                        <div class="network-state-client">
                            <div class="network-host-id">"Room ID: " {format!("{}", &conn.peer()["ripoff-robots-".len()..])}</div>
                            <div class="network-players">
                                <h3>"Players"</h3>
                                <For each={move || room_state.get().players.iter().map(|(id, name)| (id.to_owned(), name.to_owned())).collect::<Vec<_>>()}
                                    key=|(id,_name)| id.to_string()
                                    view=move |cx, (id, name)| {
                                        view!{
                                            cx, 
                                            <div class="network-player">
                                                <span class="network-player-name">{name}</span>
                                                <span class="network-player-score">{move || room_state.get().scores.get(&id).map(|x|*x).unwrap_or(0)}</span>
                                            </div>
                                        }
                                    }
                                    />
                            </div>
                            <button on:click={end_client}>"Leave"</button>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

pub fn main() {
    mount_to_body(|cx| {

        let room_state: RwSignal<RoomState> = create_rw_signal(cx, Default::default());
        let network_state = create_rw_signal(cx, NetworkState::None);
        let board = create_rw_signal(cx, Board::generate(16, 16));
        let positions = create_rw_signal(cx, board.get_untracked().initial_positions);
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
            log!("network state effect run");

            let state = network_state.get();
            match state {
                NetworkState::None => {},
                NetworkState::Server { peer, initialized: false, .. } => {
                    let _ = Reflect::set(&js_sys::global(), &"peer".into(), peer.as_ref());

                    peer.on("open", &Closure::<dyn Fn()>::new(move || {
                        log!("connection established to PeerServer")
                    }).into_js_value());

                    peer.on("error", &Closure::<dyn Fn(JsValue)>::new(move |err| {
                        console::error_1(&err);
                    }).into_js_value());

                    peer.on("close", &Closure::<dyn Fn()>::new(move || {
                        network_state.set(NetworkState::None);
                    }).into_js_value());

                    peer.on("connection", &Closure::<dyn Fn(_)>::new(move |conn: peer::DataConnection| {
                        log!("someone connected :)");

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
                                        names: vec![name.clone()],
                                        scores: vec![0],
                                    })
                                );

                                // Update network state & room state to include new player
                                conns.push(conn.clone());
                                room_state.update(|room| {
                                    room.players.insert(conn.peer(), name);
                                });
                            }
                        });

                        // Send initiating messages
                        let conn_clone = conn.clone();
                        conn.on("open", &Closure::<dyn Fn()>::new(move || {
                            // Send board state
                            peer::send(&conn_clone, 
                                &net::Message::BoardState(net::BoardStateMessage {
                                    board: board.get_untracked(),
                                })
                            );

                            // Send current list of players & scores
                            let mut ids = Vec::new();
                            let mut names = Vec::new();
                            let mut scores = Vec::new();
                            for (id, name) in room_state.get().players.iter() {
                                ids.push(id.clone());
                                names.push(name.clone());
                                scores.push(room_state.get().scores.get(id).map(|x|*x).unwrap_or(0));
                            }

                            peer::send(&conn_clone, &net::Message::PlayerJoin(net::PlayerJoinMessage {
                                ids, names, scores
                            }))
                        }).into_js_value());

                        // Broadcast `PlayerLeave` message when
                        // the player disconnects & update room state.
                        let id =conn.peer();
                        conn.on("close", &Closure::<dyn Fn()>::new(move || {
                            let id = id.clone();
                            let id_clone = id.clone();
                            network_state.update(move |state| {
                                if let NetworkState::Server { conns, .. } = state {
                                    conns.extract_if(|conn| conn.peer() == id);

                                    peer::broadcast(&conns, 
                                        &net::Message::PlayerLeave(net::PlayerLeaveMessage {
                                            id: id,
                                        })
                                    );
                                };
                            });

                            // Update room state
                            room_state.update(move |state| {
                                state.players.remove(&id_clone);
                                state.scores.remove(&id_clone);
                            })
                        }).into_js_value());
                    }).into_js_value());

                    network_state.update(|state| {
                        if let NetworkState::Server { initialized, .. } = state {
                            *initialized = true;
                        }
                    });
                },

                NetworkState::Client(ref conn) => {
                    log!("setting event for client handlers...");

                    conn.on("error", &Closure::<dyn Fn(JsValue)>::new(move |err| {
                        console::error_1(&err);
                    }).into_js_value());

                    conn.on("open", &Closure::<dyn Fn()>::new(move || {
                        log!("connection opened to host!");
                    }).into_js_value());

                    conn.on("close", &Closure::<dyn Fn()>::new(move || {
                        log!("closing connection...");
                        network_state.set(NetworkState::None);
                    }).into_js_value());

                    conn.on("data", &Closure::<dyn Fn(JsValue)>::new(move |data| {
                        let data: Result<net::Message, _> = serde_wasm_bindgen::from_value(data);
                        log!("incoming data: {:?}", &data);
                        match data {
                            Err(err) => { error!("error parsing incoming message: {:?}", err) },
                            Ok(net::Message::BoardState(state)) => {
                                board.set(state.board);
                            },
                            Ok(net::Message::PlayerJoin(msg)) => {
                                for ((id, name), score) in msg.ids.into_iter().zip(msg.names.into_iter()).zip(msg.scores.into_iter()) {
                                    room_state.update(|state| {
                                        state.players.insert(id.clone(), name);
                                        state.scores.insert(id, score);
                                    });
                                }
                            },
                            Ok(net::Message::PlayerLeave(msg)) => {
                                room_state.update(|state| { state.players.remove(&msg.id); });
                            }
                            _ => unimplemented!(),
                        }
                    }).into_js_value());
                },

                _ => {}
            }
        });


        view! { cx,  
            <Network board={board.write_only()} state={network_state} room_state={room_state} />
            <BoardWidget board={board.read_only()} positions={Some(positions)} moves={moves} />
            <MoveList moves={moves.read_only()} /> }

    })
}