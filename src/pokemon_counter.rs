use crate::utils::db_handler::{
    add_new_counter, get_row_id, get_sprite, add_counter, read_counter, save_sprite, Pokemon,
};
use crate::utils::prob_handler::{num_tries_for_x_percent_chance, probability_within_n_tries,convert_to_percentage};
use eframe::egui::Grid;
use eframe::egui::{self, CentralPanel, Response, ScrollArea, Ui, TopBottomPanel, Context, Button, Sense, SidePanel, containers::{Frame}};
use eframe::App;
use eframe::epaint::{Color32, Vec2};
use egui_extras::RetainedImage;
use std::collections::HashSet;
use std::sync::mpsc::{self, SyncSender, Receiver};
use std::thread;
use inputbot::KeybdKey;

const GEN_2_TO_5_ODDS: f64 = 8192.0;
const GEN_6_PLUS_ODDS: f64 = 4096.0;
const EGUI_KEY: egui::Key = egui::Key::Home;
const KEYDB_KEY: KeybdKey = KeybdKey::HomeKey;
const KEY_NAME: &str = "HOME";
const COUNT_BUTTON_SIZE: [f32;2] = [120.0,50.0];

pub const MIN_WINDOW_SIZE: Vec2 = Vec2::new(688.0, 524.0);

pub struct PokemonCounter {
    pokemons: Vec<Pokemon>,
    current_idx: usize,
    name_field: String,
    error_message: String,
    msg_receiver: Receiver<i32>,
    sel_gen: Generation,
    gen4_mod: Gen4Modifiers,
    gen5_mod: Gen5Modifiers,
    gen6_mod: Gen6Modifiers,
    gen7_mod: Gen7Modifiers,
    gen8_mod: Gen8Modifiers,
    odds: f64,
    cur_odds: f64
}

#[derive(Debug, Eq, PartialEq)]
pub enum Generation {
    Gen2or3,
    Gen4,
    Gen5,
    Gen6,
    Gen7,
    Gen8,
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Gen4Modifiers {
    masuda: bool,
    radar: bool,
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Gen5Modifiers {
    masuda: bool,
    charm: bool,
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Gen6Modifiers {
    masuda: bool,
    radar: bool,
    charm: bool,
    safari: bool,
    fishing: bool,
    hidden: bool,
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Gen7Modifiers {
    masuda: bool,
    charm: bool,
    sos: bool,
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Gen8Modifiers {
    masuda: bool,
    charm: bool,
    radar: bool,
    dynamax: bool,
    underground: bool,
    mass: bool,
    massive: bool,
    dex10: bool,
    dexmax: bool
}


impl App for PokemonCounter {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        TopBottomPanel::bottom("footer").show(ctx, |ui|{
            ui.vertical_centered(|ui| {
                ui.add_space(10.);
                ui.colored_label(Color32::RED, self.error_message.clone());
            });
        });
        SidePanel::left("Pokemonlist").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for (i, pkmn) in self.pokemons.iter().enumerate() {
                    if ui
                        .button(format!("{:?}: {:?}", &pkmn.name, &pkmn.counter))
                        .clicked()
                    {
                        self.current_idx = i;
                    };
                }
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.counter_button(ui);
                self.current_label(ui);
                self.current_sprite(ui);
            });

            ui.label(format!("Current hotkey: '{}'", KEY_NAME));
            self.add_new_text_field(ui);
            self.select_generation_dropdown(ui);
            self.odds_calculator(ui);
        });
        if ctx.input().key_pressed(EGUI_KEY) {
            self.update_counter(1);
        }
        self.receive_increments();
    }
}

impl PokemonCounter {
    pub fn new(cc: &eframe::CreationContext<'_>, pokemons: Vec<Pokemon>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let (tx, rx) = mpsc::sync_channel(10);
        let c = cc.egui_ctx.clone();
        PokemonCounter::spawn_input_bot_thread(c, tx, KEYDB_KEY);

        Self {
            pokemons,
            current_idx: 0,
            name_field: "".to_owned(),
            error_message: "".to_owned(),
            msg_receiver: rx,
            sel_gen: Generation::Gen2or3,
            gen4_mod: Gen4Modifiers::default(),
            gen5_mod: Gen5Modifiers::default(),
            gen6_mod: Gen6Modifiers::default(),
            gen7_mod: Gen7Modifiers::default(),
            gen8_mod: Gen8Modifiers::default(),
            odds: 0.0,
            cur_odds: 0.0,
        }
    }

    fn spawn_input_bot_thread(c: Context, tx: SyncSender<i32>, key: KeybdKey) {
        thread::spawn(move || {
            key.unbind();
            key.block_bind(move || {
                match tx.send(1) {
                    Ok(_) => {
                        println!("Sending succeeded");
                        c.request_repaint();
                    },
                    Err(_) => println!("Failed sending"),
                };
                println!("Pressed");
            });
            inputbot::handle_input_events();
        });
    }

    fn receive_increments(&mut self) {
        let val = match self.msg_receiver.try_recv() {
            Ok(v) => v,
            Err(_) => return,
        };
        self.update_counter(val);
    }

    fn was_submitted(re: &Response) -> bool {
        re.lost_focus() && re.ctx.input().key_pressed(egui::Key::Enter)
    }

    fn get_current_mut(&mut self) -> Option<&mut Pokemon> {
        return self.pokemons.get_mut(self.current_idx);
    }

    fn get_current(&self) -> Option<&Pokemon> {
        return self.pokemons.get(self.current_idx);
    }

    fn add_new_pokemon(&mut self) {
        match add_new_counter(&self.name_field) {
            Ok(_) => {}
            Err(_) => {self.error_message = "Failed to add new counter".to_owned();},
        }
        let id = match get_row_id(&self.name_field) {
            Ok(r) => r,
            Err(_) => {
                self.error_message = "Failed to reach DB".to_owned();
                0
            }
        };
        let res = read_counter(id);
        match res {
            Ok(v) => {
                self.pokemons.push(v);
                self.current_idx = self.pokemons.len() - 1;
            }
            Err(_) => {self.error_message = "Failed to read pokemon".to_owned();},
        };
    }

    fn counter_button(&mut self, ui: &mut Ui) {
        let button = ui.add_sized(COUNT_BUTTON_SIZE, Button::new("Count"));
        if button.clicked() {
           self.update_counter(1);
        };
    }
    fn update_counter(& mut self, amount: i32) {
        let current = self.get_current_mut();
        match current {
            Some(v) => {
                v.update_counter(add_to_counter(v.id, amount));
            }
            None => {}
        }
    }
    fn add_new_text_field(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Enter Pokemon Name");
            let re = ui.text_edit_singleline(&mut self.name_field);
            if ui.button("Add new").clicked() || PokemonCounter::was_submitted(&re) {
                self.add_new_pokemon();
            };
        });
    }
    fn select_generation_dropdown(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Select the generation ");
            egui::ComboBox::from_id_source("generation-selector")
                .width(128.0)
                .selected_text(format!("{:?}", self.sel_gen))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sel_gen, Generation::Gen2or3, "Generation 2 or 3");
                    ui.selectable_value(&mut self.sel_gen, Generation::Gen4, "Generation 4");
                    ui.selectable_value(&mut self.sel_gen, Generation::Gen5, "Generation 5");
                    ui.selectable_value(&mut self.sel_gen, Generation::Gen6, "Generation 6");
                    ui.selectable_value(&mut self.sel_gen, Generation::Gen7, "Generation 7");
                    ui.selectable_value(&mut self.sel_gen, Generation::Gen8, "Generation 8");
                }
            );
        });
    }
    fn odds_calculator(&mut self, ui: &mut Ui) {
        ui.separator();
        if self.get_current().is_none() { return;}
            let counter = self.get_current().unwrap().counter;
            match self.sel_gen {
                Generation::Gen2or3 => {
                    self.gen2_calculator(ui);
                },
                Generation::Gen4 => {
                    self.gen4_calculator(ui);
                },
                Generation::Gen5 => {
                    self.gen5_calculator(ui);
                }
                Generation::Gen6 => {
                    self.gen6_calculator(ui);
                }
                Generation::Gen7 => {
                    self.gen7_calculator(ui);
                }
                Generation::Gen8 => {
                    self.gen8_calculator(ui);
                }
                _ => {ui.label("Generation not implemented!");}
            }
            
            ui.separator();
            Frame::none().fill(egui::Color32::from_gray(24)).show(ui, |ui| {
            ui.label(format!("The odds are: {:.3}%", convert_to_percentage(self.odds)));
            ui.label(format!("Your odds are: {:.3}%", convert_to_percentage(self.cur_odds)));
            ui.label(format!("50% chance within: {} tries", num_tries_for_x_percent_chance(0.5, self.cur_odds)));
            ui.label(format!("75% chance within: {} tries", num_tries_for_x_percent_chance(0.75, self.cur_odds)));
            ui.label(format!("99% chance within: {} tries", num_tries_for_x_percent_chance(0.99, self.cur_odds)));
            ui.label(format!("Current chance: {:.3}%", 
                convert_to_percentage(probability_within_n_tries(counter, self.cur_odds))
            ));
            ui.allocate_exact_size(
                Vec2::new(ui.available_width(), ui.available_height()), 
                Sense::hover());
            });
    }

    fn gen2_calculator(&mut self, ui: &mut Ui) {
        self.odds = 1.0/GEN_2_TO_5_ODDS;
        self.cur_odds = self.odds;
    }
    fn gen4_calculator(&mut self, ui: &mut Ui) {
        let mut clicked = false;
        self.odds = 1.0/GEN_2_TO_5_ODDS;
        self.cur_odds = self.calc_gen4_odds();
        ui.label("Special Methods");
        Grid::new("gen6-grid").show(ui, |ui| {
            if ui.checkbox(&mut self.gen4_mod.masuda, "Masuda Method").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen4_mod.radar, "Poké Radar chaining >40").clicked() {clicked = true}
        });
        if clicked {self.cur_odds = self.calc_gen4_odds();}
    }
    fn gen5_calculator(&mut self, ui: &mut Ui) {
        let mut clicked = false;
        self.odds = 1.0/GEN_2_TO_5_ODDS;
        self.cur_odds = self.calc_gen5_odds();
        ui.label("Special Methods");
        Grid::new("gen6-grid").show(ui, |ui| {
            if ui.checkbox(&mut self.gen5_mod.masuda, "Masuda Method").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen5_mod.charm, "Shiny Charm").clicked() {clicked = true}
        });
        if clicked {self.cur_odds = self.calc_gen5_odds();}
    }
    fn gen6_calculator(&mut self, ui: &mut Ui) {
        let mut clicked = false;
        self.odds = 1.0/GEN_6_PLUS_ODDS;
        self.cur_odds = self.calc_gen6_odds();
        ui.label("Special Methods");
        Grid::new("gen6-grid").show(ui, |ui| {
            if ui.checkbox(&mut self.gen6_mod.masuda, "Masuda Method").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen6_mod.charm, "Shiny Charm").clicked() {clicked = true}
            ui.end_row();
            if ui.checkbox(&mut self.gen6_mod.radar, "Poké Radar chaining >40").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen6_mod.safari, "Friend Safari").clicked() {clicked = true}
            ui.end_row();
            if ui.checkbox(&mut self.gen6_mod.fishing, "Consecutive fishing: ≥20").clicked() {clicked = true}
            //ui.checkbox(&mut self.gen6_mod.hidden, "Hidden Pokémon: Search Level 200 + X")//.clicked() {clicked = true}
        });
        
        if clicked {self.cur_odds = self.calc_gen6_odds();}
    }
    fn gen7_calculator(&mut self, ui: &mut Ui) {
        let mut clicked = false;
        self.odds = 1.0/GEN_6_PLUS_ODDS;
        self.cur_odds = self.calc_gen7_odds();
        ui.label("Special Methods");
        Grid::new("gen6-grid").show(ui, |ui| {
            if ui.checkbox(&mut self.gen7_mod.masuda, "Masuda Method").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen7_mod.charm, "Shiny Charm").clicked() {clicked = true}
            ui.end_row();
            if ui.checkbox(&mut self.gen7_mod.sos, "SOS Battles: ≥31").clicked() {clicked = true}
        });
        if clicked {self.cur_odds = self.calc_gen7_odds();}
    }
    fn gen8_calculator(&mut self, ui: &mut Ui) {
        let mut clicked = false;
        self.odds = 1.0/GEN_6_PLUS_ODDS;
        self.cur_odds = self.calc_gen8_odds();
        ui.label("Special Methods");
        Grid::new("gen6-grid").show(ui, |ui| {
            if ui.checkbox(&mut self.gen8_mod.masuda, "Masuda Method").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen8_mod.charm, "Shiny Charm").clicked() {clicked = true}
            ui.end_row();
            if ui.checkbox(&mut self.gen8_mod.radar, "Poké Radar chaining: ≥40").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen8_mod.dynamax, "Dynamax Adventure").clicked() {clicked = true}
            ui.end_row();
            if ui.checkbox(&mut self.gen8_mod.mass, "Mass outbreak").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen8_mod.massive, "Massive mass outbreak").clicked() {clicked = true}
            ui.end_row();
            if ui.checkbox(&mut self.gen8_mod.dex10, "Pokédex research level 10").clicked() {clicked = true}
            if ui.checkbox(&mut self.gen8_mod.dexmax, "Pokédex research  level max").clicked() {clicked = true}
        });
        if ui.checkbox(&mut self.gen8_mod.underground, "Grand Underground, after 'something good happens'").clicked() {clicked = true}
        if clicked {self.cur_odds = self.calc_gen8_odds();}
    }

    fn current_label(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let current = self.get_current();
            ui.label("Currently loaded:");
            match current {
                Some(v) => {
                    ui.label(format!("{:?}", v.name));
                    ui.label(format!("Resets: {:?}", v.counter));
                }
                None => {}
            };
        });
    }
    fn current_sprite(&mut self, ui: &mut Ui) {
        let current_name = self.get_current().unwrap().name.clone();
        let path = generate_sprite_path(&current_name);

        let img_bytes = match get_sprite(&current_name) {
            Ok(r) => r,
            Err(_) => {
                let img_bytes = match reqwest::blocking::get(path.to_ascii_lowercase()) {
                    Ok(r) => match r.bytes() {
                        Ok(r) => r,
                        Err(e) => {
                            self.error_message = format!("{:?}", e.to_string());
                            return;
                        }
                    },
                    Err(e) => {
                        self.error_message = format!("{:?}", e.to_string());
                        return;
                    }
                };
                match save_sprite(&current_name, img_bytes) {
                    Ok(b) => b,
                    Err(_) => return,
                }
            }
        };
        let image = match RetainedImage::from_image_bytes(current_name, &img_bytes) {
            Ok(r) => r,
            Err(_) => {
                match RetainedImage::from_image_bytes(
                    "unknown",
                    include_bytes!("../img/unknown.png"),
                ) {
                    Ok(r) => r,
                    Err(_) => {
                        self.error_message = "Error loading sprite".to_owned();
                        return;
                    }
                }
            }
        };
        image.show(ui);
    }

    fn get_gen4_modifier(&self) -> i32 {
        let mut total_mod = 1;
        if self.gen4_mod.masuda {total_mod += 4}
        if self.gen4_mod.radar {total_mod += 40}
        return total_mod;
    }
    fn calc_gen4_odds(&self) -> f64 {
        self.get_gen4_modifier() as f64 / GEN_2_TO_5_ODDS
    }

    fn get_gen5_modifier(&self) -> i32 {
        let mut total_mod = 1;
        if self.gen5_mod.masuda {total_mod += 5}
        if self.gen5_mod.charm {total_mod += 2}
        return total_mod;
    }
    fn calc_gen5_odds(&self) -> f64 {
        self.get_gen5_modifier() as f64 / GEN_2_TO_5_ODDS
    }

    fn get_gen6_modifier(&self) -> i32 {
        let mut total_mod = 1;
        if self.gen6_mod.masuda {total_mod += 5}
        if self.gen6_mod.radar {total_mod += 81}
        if self.gen6_mod.charm {total_mod += 2}
        if self.gen6_mod.safari {total_mod += 4}
        if self.gen6_mod.fishing {total_mod += 40}
        //TODO impl hidden pokemon calculation
        return total_mod;
    }
    fn calc_gen6_odds(&self) -> f64 {
        self.get_gen6_modifier() as f64 / GEN_6_PLUS_ODDS
    }

    fn get_gen7_modifier(&self) -> i32 {
        let mut total_mod = 1;
        if self.gen7_mod.masuda {total_mod += 5}
        if self.gen7_mod.charm {total_mod += 2}
        if self.gen7_mod.sos {total_mod += 12}
        return total_mod;
    }
    fn calc_gen7_odds(&self) -> f64 {
        self.get_gen7_modifier() as f64 / GEN_6_PLUS_ODDS
    }

    fn get_gen8_modifier(&self) -> i32 {
        let mut total_mod = 1;
        if self.gen8_mod.masuda {total_mod += 5}
        if self.gen8_mod.charm {total_mod += 2}
        if self.gen8_mod.radar {total_mod += 40}
        if self.gen8_mod.dynamax {total_mod += 13}
        if self.gen8_mod.underground {total_mod += 1}
        if self.gen8_mod.mass {total_mod += 25}
        if self.gen8_mod.massive {total_mod += 12}
        if self.gen8_mod.dex10 {total_mod += 1}
        if self.gen8_mod.dexmax {total_mod += 2}
        return total_mod;
    }
    fn calc_gen8_odds(&self) -> f64 {
        self.get_gen8_modifier() as f64 / GEN_6_PLUS_ODDS
    }
}

fn add_to_counter(current_id: i32, amnt: i32) -> i32 {
    if current_id == -1 {
        println!("No counter loaded");
        return -1;
    }
    let res = add_counter(current_id, amnt);
    match res {
        Ok(count) => {
            println!("{:?}", count);
            return count;
        }
        Err(_) => {
            println!("Error, could not increment.");
            return -1;
        }
    }
}

fn generate_sprite_path(loaded_pkmn: &str) -> String {
    let mut path: String = "https://play.pokemonshowdown.com/sprites/dex/".to_owned();
    path.push_str(&loaded_pkmn);
    path.push_str(".png");
    return path;
}
