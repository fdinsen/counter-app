#![windows_subsystem = "windows"]

mod utils;
use inputbot::KeybdKey::ScrollLockKey;
use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};
use std::sync::{Arc, Mutex};
use std::thread;
use utils::cli_handler::{handle_input, increment, run_cli, State};
use utils::db_handler::{add_new_counter, connect, get_all_counters, get_row_id, read_counter, save_sprite, get_sprite};

slint::slint! {
    import { VerticalBox, Button, HorizontalBox, ListView, ListView ,LineEdit} from "std-widgets.slint";

    export struct slintCounter := {
        id: int, name: string, counter: int,
    }

    HelloWorld := Window{
        title: "Counter Test App";
        width: 400px;
        height: 420px;
        property <int> counter: 0;
        property <int> loaded-counter-id: -1;
        property <string> loaded-counter-name: "";
        property <string> add-new-counter-text: "Enter name";
        property <string> error-msg: "";
        property <[slintCounter]> counters-list : [];
        property <image> image : @image-url("img/unknown.png");
        callback add-counter <=> fakesavebtn.clicked;
        callback count <=> button.clicked;
        callback loaded <=> fakeloadbtn.clicked;
        callback refresh <=> refreshbutton.clicked;
        HorizontalBox {
            width: 400px;
            height: 100px;
            button := Button {
                text: "Counter: " + counter;
                clicked => {
                    counter += 1;
                }
            }
            Text {
                text: "Loaded Counter: " + loaded-counter-name;
                color: green;
            }
            img := Image {
                width: 100px;
                source: image;
                image-fit: fill;
            }
        }
        HorizontalBox {
            y: 100px;
            width: 400px;
            height: 30px;
            addNewBox := LineEdit{
                text: add-new-counter-text;
                // vertical-alignment: center;
                // single-line: true;
                height: 30px;
                edited => {
                    add-new-counter-text = text;
                }
                accepted => {
                    addNewBox.text = "Enter name";
                    add-counter();
                    loaded();
                }
            }
            savebutton := Button {
                height: 30px;
                width: 100px;
                text: "Add new";
                clicked => {
                    addNewBox.text = "Enter name";
                    add-counter();
                    loaded();
                }
            }
            refreshbutton := Button {
                height: 30px;
                width: 80px;
                text: "Refresh";
            }
        }
        fakeloadbtn := Button {
            width: 0px;
            height: 0px;
            y: 0px;
        }
        fakesavebtn := Button {
            width: 0px;
            height: 0px;
        }
        ListView {
            width: 400px;
            height: 250px;
            y: 150px;
            for data in counters-list
                : Button {
                    height: 30px;
                    color: white;
                    // text: data.name + ": " + data.counter;
                    Text {
                        x: 5px;
                        height: 30px;
                        text: data.name + ": " + data.counter;
                        color: black;
                        vertical-alignment: center;
                    }
                    clicked => {loaded-counter-id = data.id; counter = data.counter; loaded-counter-name = data.name; loaded();}
                }
        }
        Text {
            y: 400px;
            color: red;
            text: error-msg;
        }
    }
}

macro_rules! fetch_counters {
    ($window: expr) => {{
        let c = get_all_counters();
        match c {
            Ok(l) => {
                let v = VecModel::from(
                    l.iter()
                        .map(|x| slintCounter {
                            id: x.id,
                            name: slint::SharedString::from(&x.name),
                            counter: x.counter,
                        })
                        .collect::<Vec<slintCounter>>(),
                );
                $window.set_counters_list(ModelRc::new(v));
            }
            Err(_) => {
                set_error!($window, "Failed to load counters.")
            }
        }
    }};
}

macro_rules! add_counter {
    ($window: expr) => {{
        let value = $window.get_add_new_counter_text();
        match add_new_counter(&value) {
            Ok(_) => {}
            Err(_) => set_error!($window, "Failed to add new counter"),
        }
        let id = match get_row_id(&value) {
            Ok(r) => r,
            Err(_) => {
                set_error!($window, "Failed to reach DB");
                return;
            }
        };
        let res = read_counter(id);
        match res {
            Ok(r) => {
                $window.set_loaded_counter_id(id);
                $window.set_counter(r.counter);
                $window.set_loaded_counter_name(slint::SharedString::from(&r.name));
                fetch_counters!($window);
                println!("Loaded counter {:?}, current count: {:?}", value, r.counter);
            }
            Err(_) => {
                println!("Counter with name {:?} does not exist.", value);
                set_error!($window, "Counter with given name does not exist.")
            }
        };
    }};
}

macro_rules! update_counter {
    ($window: expr, $do_increment: expr) => {{
        let value = $window.get_loaded_counter_id();
        if($do_increment) {increment(value);}
        let res = read_counter(value);
        match res {
            Ok(r) => $window.set_counter(r.counter),
            Err(_) => {
                println!("Error reading counter.");
                set_error!($window, "Error reading counter.");
            }
        };
    }};
}

macro_rules! set_error {
    ($window: expr, $msg: expr) => {{
        $window.set_error_msg(slint::SharedString::from($msg));
    }};
}

fn main() {
    let res = connect();
    match res {
        Ok(_) => {}
        Err(_) => {
            println!("Could not create/connect to db. Contact developer.")
        }
    }
    let cli = false;

    if cli {
        println!("Welcome to the counter app!");
        println!("Type help to get list of commands.");
        let mut id = -1;
        let mut state: State = State::Cont;

        while state != State::Exit {
            state = cliruntime(&id);
            if let State::Load(next) = state { id = next } // destructures but ignores errors
        }
    } else {
        let window = HelloWorld::new();
        let window_weaks = Arc::new(Mutex::new(window.as_weak()));
        thread::spawn(move || {
            ScrollLockKey.unbind();
            ScrollLockKey.block_bind(move || {
                let window_weak = match window_weaks.lock() {
                    Ok(r) => r,
                    Err(_) => {
                        println!("Error unpacking window.");
                        return;
                    }
                };
                window_weak.upgrade_in_event_loop(move |window| {
                    let idx = window.get_loaded_counter_id();
                    increment(idx);

                    let res = read_counter(idx);
                    match res {
                        Ok(r) => window.set_counter(r.counter),
                        Err(_) => set_error!(window, "Error reading counter"),
                    };
                })
            });
            inputbot::handle_input_events();
        });

        fetch_counters!(window);

        let window_weak = window.as_weak();
        window.on_add_counter(move || {
            let window = match window_weak.upgrade() {
                Some(v) => v,
                None => {
                    println!("Error unpacking window in add counter.");
                    return;
                }
            };
            add_counter!(window);
        });

        let window_weak = window.as_weak();
        window.on_count(move || {
            let window = match window_weak.upgrade() {
                Some(v) => v,
                None => {
                    println!("Error unpacking window in count.");
                    return;
                }
            };
            update_counter!(window, true);
        });

        let window_weak = window.as_weak();
        window.on_refresh(move || {
            let window = match window_weak.upgrade() {
                Some(v) => v,
                None => {
                    println!("Error unpacking window in refresh");
                    return;
                }
            };
            fetch_counters!(window);
            update_counter!(window, false);
        });

        let window_weak = window.as_weak();
        window.on_loaded(move || {
            let window = match window_weak.upgrade() {
                Some(v) => v,
                None => {
                    println!("Error unpacking window when loading.");
                    return;
                }
            };
            let loaded = window.get_loaded_counter_name();
            let mut path: String = "https://play.pokemonshowdown.com/sprites/dex/".to_owned();
            path.push_str(&loaded);
            path.push_str(".png");

            let img_bytes = match get_sprite(&loaded) {
                Ok(r) => r,
                Err(_) => {
                    let img_bytes = match reqwest::blocking::get(path.to_ascii_lowercase()) {
                        Ok(r) => match r.bytes() {
                            Ok(r) => r,
                            Err(e) => {set_error!(window, e.to_string()); return; },
                        },
                        Err(e) => {set_error!(window, e.to_string()); return; },
                    };
                    match save_sprite(&loaded,img_bytes) {
                        Ok(b) => b,
                        Err(e) => return,
                    }
                }
            };
            
            let mut cat_image = match image::load_from_memory(&img_bytes) {
                Ok(r) => r.into_rgba8(),
                Err(_) => match image::open("img/unknown.png") {
                    Ok(r) => r.into_rgba8(),
                    Err(_) => {
                        set_error!(window, "Error loading sprite.");
                        return;
                    }
                },
            };

            image::imageops::colorops::brighten_in_place(&mut cat_image, 20);

            let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                cat_image.as_raw(),
                cat_image.width(),
                cat_image.height(),
            );
            let image = Image::from_rgba8(buffer);
            window.set_image(image);
        });

        window.run();
    }
}

fn cliruntime(current_id: &i32) -> State {
    let id = *current_id;
    let clonedid = id;
    thread::spawn(move || {
        ScrollLockKey.unbind();
        ScrollLockKey.block_bind(move || {
            let id2 = clonedid;
            println!();
            increment(id2);
        });

        inputbot::handle_input_events();
    });

    run_cli!(id)
}