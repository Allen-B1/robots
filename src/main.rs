use std::collections::HashSet;

use leptos::*;
mod rand;

#[derive(Clone)]
struct Board {
    pub width: usize,

    /// A width * (height - 1) size array.
    /// There is a wall between (i, j) and (i, j+1) iff
    /// horizontal_walls[i, j] is true.
    pub horizontal_walls: Vec<bool>,

    /// A (width - 1) * height size array.
    /// There is a wall between (i, j) and (i+1, j) iff
    /// verticall_walls[i, j] is true.
    pub vertical_walls: Vec<bool>,

    pub robots: [usize; 5]
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

    pub fn empty(width: usize, height: usize) -> Self {
        let mut board = Board {
            width,
            horizontal_walls: vec![false; width * (height - 1)],
            vertical_walls: vec![false; (width - 1) * height],
            robots: [0, 1, 2, 3, 4]
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

        board
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
pub fn BoardWidget(cx: Scope, bouncy: bool) -> impl IntoView {
    let (board, set_board) = create_signal::<Board>(cx, Board::empty(16, 16));
    let (active, set_active)  =create_signal(cx, Option::<usize>::None);

    view !{
        cx, 
        <div class="board" style={format!("width:{}px;height:{}px", 32 * board.get().width, 32 * board.get().height())}>
            <For 
                each=move || 0..board.get().width*board.get().height()
                key=|&i| i
                view=move |cx, i| {
                    view! {
                        cx, 
                        <div class="tile" style={
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
                        <div class={move || format!("robot robot-{} {}", i, if active.get() == Some(i) { "active" } else { "" })}
                            on:click=move |_| set_active.set(Some(i))
                            style={
                            let width = board.get().width;
                            let pos = board.get().robots[i];
                            format!("top:{}px;left:{}px", 32 * (pos/width), 32 * (pos%width))
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
    mount_to_body(|cx| view! { cx,  <BoardWidget bouncy={false} /> })
}