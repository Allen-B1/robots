use wasm_bindgen::JsValue;

pub trait IntoJsValueRef<'a, T> {
    fn into_ref(self, ptr: &'a mut JsValue) -> &'a JsValue
        where Self: Sized;
}

impl <'a, V> IntoJsValueRef<'a, V> for V
    where V: Into<JsValue>
{
    fn into_ref(self, ptr: &'a mut JsValue) -> &'a JsValue {
        let _ = std::mem::replace(ptr, self.into());
        ptr
    }
}

/* 
impl<'a, R> IntoJsValueRef<'a, R> for &'a R
    where R: AsRef<JsValue> {
    fn into_ref(self, _: &'a mut JsValue) -> &'a JsValue {
        self.as_ref()
    }
}*/

#[macro_export]
macro_rules! object {
    ($($k:expr => $v:expr),*) => {
        {
            let object = ::js_sys::Object::new();
            $(
                match unsafe { ::js_sys::Reflect::set(
                    <::js_sys::Object as ::core::convert::AsRef<::wasm_bindgen::JsValue>>::as_ref(&object),
                    &::wasm_bindgen::JsValue::from_str($k),
                    &::std::convert::Into::<::wasm_bindgen::JsValue>::into($v),
                ) } {
                    ::std::result::Result::Ok(_) => {},
                    ::std::result::Result::Err(err) => { dbg!("error constructing JS object: {:?}", err); }
                }
            )*
            object
        }
    }
}

#[macro_export]
macro_rules! array {
    ($($v:expr),*) => {
        {
            let array = ::js_sys::Array::new();
            $(
                ::js_sys::Array::push(&array, 
                    &::std::convert::Into::<::wasm_bindgen::JsValue>::into($v),
                );
            )*
            array
        }
    }
}

#[macro_export]
macro_rules! set {
    ($($v:expr),*) => {
        {
            let set = ::js_sys::Set::new(&::wasm_bindgen::JsValue::NULL);
            $(
                ::js_sys::Set::add(&set, 
                    &::std::convert::Into::<::wasm_bindgen::JsValue>::into($v),
                );
            )*
            set
        }
    }
}
#[macro_export]
macro_rules! map {
    ($($k:expr => $v:expr),*) => {
        {
            let map = ::js_sys::Map::new();
            $(
                ::js_sys::Map::set(&map, 
                    &::std::convert::Into::<::wasm_bindgen::JsValue>::into($k),
                    &::std::convert::Into::<::wasm_bindgen::JsValue>::into($v),
                );
            )*
            map
        }
    }
}
