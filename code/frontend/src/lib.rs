use wasm_bindgen::prelude::*;
use web_sys::{Document, FormData, HtmlElement, HtmlInputElement, RequestInit, Request, RequestMode};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn send_file(document: &Document) {
    let files = document
        .get_element_by_id("media")
        .expect("should have media on the page")
        .dyn_ref::<HtmlInputElement>()
        .expect("hand should be an HtmlInputElement")
        .files()
        .expect("Should be able to get file list");

    let fd = FormData::new().expect("should be able to create form data");
    for file_idx in 0..files.length() {
        let possible_file = files.item(file_idx);
        if let Some(file) = possible_file{
            fd.append_with_blob_and_filename("media",&file,&file.name()).expect("should be able to add file to Form Data");
        }
    }

    wasm_bindgen_futures::spawn_local(async move {
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.body(Some(&fd));
        opts.mode(RequestMode::Cors);

        let url = format!("../api/media/v1/uploader");

        if let Ok( request ) = Request::new_with_str_and_init(&url, &opts) {
            if let Ok(_) = request.headers().set("Accept", "application/octet-stream"){


                let window = web_sys::window().unwrap();
                if let Ok( resp_value ) = JsFuture::from(window.fetch_with_request(&request)).await {
                    console_log!("Response From Upload: {:?}",resp_value);
                }
            }
        }
    });
}

fn send_delete(document: &Document) {
    let filename_to_delete = document
        .get_element_by_id("delete-name")
        .expect("should have delete-name on the page")
        .dyn_ref::<HtmlInputElement>()
        .expect("delete-name should be an HtmlInputElement")
        .value();

    wasm_bindgen_futures::spawn_local(async move {
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);

        let url = format!("../api/media/v1/deleter/{}",filename_to_delete);

        if let Ok( request ) = Request::new_with_str_and_init(&url, &opts) {
            if let Ok(_) = request.headers().set("Accept", "application/octet-stream"){


                let window = web_sys::window().unwrap();
                if let Ok( resp_value ) = JsFuture::from(window.fetch_with_request(&request)).await {
                    console_log!("Response From Upload: {:?}",resp_value);
                }
            }
        }
    });
}

fn setup() {

    let window = web_sys::window().expect("no global window exists"); 
    let document = window.document().expect("should have a document window");

    //setup uplaod button
    let handle_upload = Closure::<dyn Fn()>::new(
        move || {
            let window = web_sys::window().expect("no global window exists");
            let document = window.document().expect("should have a document window");

            send_file(&document);
        },
    );

    document
        .get_element_by_id("upload")
        .expect("should have upload on the page")
        .dyn_ref::<HtmlElement>()
        .expect("upload should be HtmlElement")
        .set_onclick(Some(handle_upload.as_ref().unchecked_ref()));

    //setup uplaod button
    let handle_delete = Closure::<dyn Fn()>::new(
        move || {
            let window = web_sys::window().expect("no global window exists");
            let document = window.document().expect("should have a document window");

            send_delete(&document);
        },
    );

    document
        .get_element_by_id("delete")
        .expect("should have delete on the page")
        .dyn_ref::<HtmlElement>()
        .expect("upload should be HtmlElement")
        .set_onclick(Some(handle_delete.as_ref().unchecked_ref()));

    handle_upload.forget();
    handle_delete.forget();
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    setup();

    Ok(())
}
