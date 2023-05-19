use std::collections::HashSet;

use leptos::*;
use leptos::ev::KeyboardEvent;
mod rand;

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
}

#[component]
pub fn SimpleCounter(cx: Scope, initial_value: i32) -> impl IntoView {
    // create a reactive signal with the initial value
    let (value, set_value) = create_signal(cx, initial_value);

    // create event handlers for our buttons
    // note that `value` and `set_value` are `Copy`, so it's super easy to move them into closures
    let clear = move |_| set_value.set(0);
    let decrement = move |_| set_value.update(|value| *value -= 1);
    let increment = move |_| set_value.update(|value| *value += 1);

    // this JSX is compiled to an HTML template string for performance
    view! {
        cx,
        <div>
            <button on:click=clear>"Clear"</button>
            <button on:click=decrement>"-1"</button>
            <span>"Value: " {move || value().to_string()} "!"</span>
            <button on:click=increment>"+1"</button>
        </div>
    }
}

#[component]
pub fn BoardWidget(cx: Scope, board: ReadSignal<Board>, positions: Option<(ReadSignal<RobotPositions>, WriteSignal<RobotPositions>)>) -> impl IntoView {
    let (positions, set_positions) = match positions {
        None => (Signal::derive(cx, move || board().initial_positions), None),
        Some((a, b)) => (a.into(), Some(b))
    };

    let keydown = move |robot, evt: KeyboardEvent| {
        log!("{}", evt.code());

        let mut positions = positions();
        let horizontal_walls = board().horizontal_walls;
        let vertical_walls = board().vertical_walls;
        let width = board().width;

        match evt.code().as_str() {
            "ArrowDown" => {
                let wall_pos = horizontal_walls.iter().enumerate()
                    .filter(|(i, b)| (**b) && (*i % width  == positions[robot] % width))
                    .map(|(i, _)| i / width)
                    .filter(|&i| i + 1 > positions[robot] / width)
                    .next().unwrap_or(board().height() - 1);
                let robot_pos = positions.iter()
                    .filter(|&i| i % width == positions[robot] % width)
                    .map(|i| i / width)
                    .filter(|&i| i > positions[robot] / width)
                    .min().map(|i| i - 1).unwrap_or(board().height() - 1);
                let target_pos = usize::min(wall_pos, robot_pos);

                positions[robot] = positions[robot] % width + target_pos * width;
                set_positions.unwrap().set(positions);
            },
            "ArrowUp" => {
                let wall_pos = horizontal_walls.iter().enumerate().rev()
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
                set_positions.unwrap().set(positions);
            },
            "ArrowRight" => {
                let wall_pos = vertical_walls.iter().enumerate()
                    .filter(|&(i, b)| (*b && i / (width - 1) == positions[robot] / width))
                    .map(|(i, _)| i % (width - 1))
                    .filter(|&i| i + 1 > positions[robot] % width)
                    .next().map(|i| i).unwrap_or(board().width - 1);
                let robot_pos = positions.iter()
                    .filter(|&i| i / width == positions[robot] / width)
                    .map(|i| i % width)
                    .filter(|&i| i > positions[robot] % width)
                    .min().map(|i| i - 1).unwrap_or(board().width - 1);
                let target_pos = usize::min(wall_pos, robot_pos);

                positions[robot] = (positions[robot] / width) * width + target_pos;
                set_positions.unwrap().set(positions);
            },
            "ArrowLeft" => {
                let wall_pos = vertical_walls.iter().enumerate().rev()
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
                set_positions.unwrap().set(positions);
            }
            _ => {}
        }
    };

    view !{
        cx, 
        <div class="board" style={format!("width:{}px;height:{}px", 32 * board.get().width, 32 * board.get().height())}>
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

pub fn main() {
    mount_to_body(|cx| {
        let (board, set_board) = create_signal(cx, Board::generate(16, 16));
        let (positions, set_positions) = create_signal(cx, board().initial_positions);
        view! { cx,  <BoardWidget board={board} positions={Some((positions, set_positions))} /> }
    })
}