use std::borrow::Borrow;

use js_sys::{Object, Function, Reflect, Array};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object)]
    #[derive(Clone)]
    pub type DataConnection;

    #[wasm_bindgen(method)]
    pub fn send(this: &DataConnection, data: &JsValue);

    #[wasm_bindgen(method)]
    pub fn close(this: &DataConnection);

    #[wasm_bindgen(method)]
    pub fn on(this: &DataConnection, event: &str, callback: &JsValue);

    #[wasm_bindgen(method, getter)]
    pub fn peer(this: &DataConnection) -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object)]
    #[derive(Clone)]
    pub type Peer;

    #[wasm_bindgen(constructor)]
    pub fn new(id: &str, options: &JsValue) -> Peer;

    #[wasm_bindgen(method)]
    pub fn connect(this: &Peer, id: &str, options: &JsValue) -> DataConnection;

    #[wasm_bindgen(method)]
    pub fn on(this: &Peer, event: &str, callback: &JsValue);

    #[wasm_bindgen(method)]
    pub fn disconnect(this: &Peer);

    #[wasm_bindgen(method)]
    pub fn destroy(this: &Peer);

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Peer) -> String;
}
