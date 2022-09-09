mod utils;
use inputbot::KeybdKey::ScrollLockKey;
use slint::{ModelRc, VecModel};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::Duration;
use utils::cli_handler::{handle_input, increment, run_cli, State};
use utils::db_handler::{
    add_new_counter, connect, get_all_counters, get_row_id, read_counter, Counter,
};

slint::slint! {
    import { VerticalBox, Button, HorizontalBox, ListView, ListView ,LineEdit} from "std-widgets.slint";

    export struct slintCounter := {
        id: int, name: string, counter: int,
    }

    HelloWorld := Window{
        title: "Counter Test App";
        width: 400px;
        height: 400px;
        property <int> counter: 0;
        property <int> loaded-counter-id: -1;
        property <string> loaded-counter-name: "";
        property <string> add-new-counter-text: "Enter name";
        property <[slintCounter]> counters-list : [];
        callback add-counter <=> savebutton.clicked;
        callback count <=> button.clicked;
        callback loaded <=> fakebtn.clicked;
        HorizontalBox {
            width: 100px;
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
                    add-counter();
                }
            }
            savebutton := Button {
                height: 30px;
                width: 100px;
                text: "Add new";
            }
            errorText := Text {
                text: "";
                color: red;
            }
        }
        fakebtn := Button {
            width: 400px;
            height: 250px;
            y: 150px;
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
                    clicked => {loaded-counter-id = data.id; counter = data.counter; loaded-counter-name = data.name;}
                }
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
            Err(_) => {}
        }
    }};
}

macro_rules! add_counter {
    ($window: expr) => {{
        let value = $window.get_add_new_counter_text();
        add_new_counter(&value);
        let id = get_row_id(&value).unwrap();
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
            }
        };
    }};
}

macro_rules! increment_counter {
    ($window: expr) => {{
        let value = $window.get_loaded_counter_id();
        increment(value);
        let res = read_counter(value);
        match res {
            Ok(r) => $window.set_counter(r.counter),
            Err(_) => println!("Error reading counter"),
        };
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
            match state {
                State::Load(next) => id = next,
                _ => {}
            }
        }
    } else {
        let mut id = -1;

        let window = HelloWorld::new();
        let window_weaks = Arc::new(Mutex::new(window.as_weak()));
        thread::spawn(move || {
            ScrollLockKey.unbind();
            ScrollLockKey.block_bind(move || {
                println!("pressed.");
                let window_weak = window_weaks.lock().unwrap();
                window_weak.upgrade_in_event_loop(move |window| {
                    println!("In loop.");
                    let idx = window.get_loaded_counter_id();
                    increment(idx);
    
                    let res = read_counter(idx);
                    match res {
                        Ok(r) => window.set_counter(r.counter),
                        Err(_) => println!("Error reading counter"),
                    };
                })
                
            });
            inputbot::handle_input_events();
            
        });

        // thread::spawn(move || {
        //     let win_weak = window_weak.to_owned();
        //     while true {
        //         let win = win_weak.upgrade();
        //         let win = match win {
        //             Some(w) => w,
        //             None => {println!("empty window."); continue;},
        //         };
        //         let loadedid = win.get_loaded_counter_id();
        //         if loadedid != id {
        //             id = loadedid;
        //         }
        //         let res = read_counter(id);
        //         match res {
        //             Ok(r) => win.set_counter(r.counter),
        //             Err(_) => println!("Error reading counter"),
        //         };
        //         println!("Test");
        //         thread::sleep(Duration::from_millis(10));
        //     }
        // });

        fetch_counters!(window);
        
        let window_weak = window.as_weak();
        window.on_add_counter(move || {
            let window = window_weak.upgrade().unwrap();
            add_counter!(window);
        });

        let window_weak = window.as_weak();
        window.on_count(move || {
            let window = window_weak.upgrade().unwrap();
            increment_counter!(window);
        });

        window.run();
    }
}

fn cliruntime(current_id: &i32) -> State {
    let id = current_id.clone();
    let clonedid = id.clone();
    thread::spawn(move || {
        ScrollLockKey.unbind();
        ScrollLockKey.block_bind(move || {
            let id2 = clonedid.clone();
            println!("");
            increment(id2);
        });

        inputbot::handle_input_events();
    });

    run_cli!(id)
}
