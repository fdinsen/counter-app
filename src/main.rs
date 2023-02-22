// #![windows_subsystem = "windows"]

mod utils;
mod pokemon_counter;

use eframe::epaint::Vec2;
use inputbot::KeybdKey;
use std::thread;
use utils::cli_handler::{handle_input, increment, run_cli, State};
use utils::db_handler::{connect, get_all_counters};
use pokemon_counter::{PokemonCounter, MIN_WINDOW_SIZE};
use eframe::{run_native, NativeOptions};

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
        let pokemons = match get_all_counters() {
            Ok(r) => r,
            Err(_) => {
                println!("Failed");
                panic!("Failed");
            }
        };
        let mut win_option = NativeOptions::default();
        win_option.min_window_size = Some(MIN_WINDOW_SIZE);

        run_native("Pokemon Counter", win_option,Box::new(|cc| Box::new(PokemonCounter::new(cc, pokemons))));
    }
}

fn cliruntime(current_id: &i32) -> State {
    let id = *current_id;
    let clonedid = id;
    thread::spawn(move || {
        KeybdKey::ScrollLockKey.unbind();
        KeybdKey::ScrollLockKey.block_bind(move || {
            let id2 = clonedid;
            println!();
            increment(id2);
        });

        inputbot::handle_input_events();
    });

    run_cli!(id)
}