use eframe::egui;
use enigo::{Button, Direction::Click, Enigo, Mouse, Settings};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

static A_SECOND: u64 = 1_000_000_000; // nanoseconds of a second
static A_MILLISEC: u64 = 1_000_000; // nanoseconds of a millisecond
static DEFAULT_CPS: u64 = 60;

static FONT: &[u8] = include_bytes!("./NotoSansJP-Regular.otf");

#[derive(PartialEq, Clone, Copy)]
enum State {
    Nano,
    Milli,
    Sec,
    Cps,
}

pub struct AutoShooter {
    input: String,
    is_valid_input: bool,
    state: State,
    calculated_from: State,
    cps: u64,
    wait: u64,
    is_running: Arc<AtomicBool>,
}

impl Default for AutoShooter {
    fn default() -> Self {
        Self {
            input: DEFAULT_CPS.to_string(),
            is_valid_input: true,
            state: State::Cps,
            calculated_from: State::Cps,
            cps: DEFAULT_CPS,
            wait: A_SECOND / DEFAULT_CPS,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl AutoShooter {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert("my_font".to_owned(), egui::FontData::from_static(FONT));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        Self::default()
    }
}

impl eframe::App for AutoShooter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("自動連射ツール");
            if ui
                .radio_value(&mut self.state, State::Cps, "CpS指定")
                .clicked()
            {
                self.input = self.cps.to_string();
            };
            if ui
                .radio_value(&mut self.state, State::Nano, "ナノ秒指定")
                .clicked()
            {
                self.input = self.wait.to_string();
            };
            if ui
                .radio_value(&mut self.state, State::Milli, "ミリ秒指定")
                .clicked()
            {
                self.input = (self.wait / A_MILLISEC).to_string();
            };
            if ui
                .radio_value(&mut self.state, State::Sec, "秒指定")
                .clicked()
            {
                self.input = (self.wait / A_SECOND).to_string();
            };

            ui.horizontal(|ui| {
                let input_label = ui.add(match self.state {
                    State::Nano => egui::Label::new("連射間隔 (ns): "),
                    State::Milli => egui::Label::new("連射間隔 (ms): "),
                    State::Sec => egui::Label::new("連射間隔 (s): "),
                    State::Cps => egui::Label::new("CpS: "),
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::LEFT), |ui| {
                    let input_text = ui
                        .add(egui::TextEdit::singleline(&mut self.input).desired_width(100.))
                        .labelled_by(input_label.id);

                    if input_text.changed() {
                        let parsed_res = self.input.parse::<u64>();
                        self.is_valid_input = parsed_res.is_ok();

                        if let Ok(parsed) = parsed_res {
                            self.calculated_from = self.state;
                            match self.state {
                                State::Nano => {
                                    self.wait = parsed;
                                    self.cps = A_SECOND / self.wait;
                                }
                                State::Milli => {
                                    self.wait = parsed * (A_SECOND / A_MILLISEC);
                                    self.cps = A_SECOND / self.wait;
                                }
                                State::Sec => {
                                    self.wait = parsed * A_SECOND;
                                    self.cps = A_SECOND / self.wait;
                                }
                                State::Cps => {
                                    self.cps = parsed;
                                    self.wait = A_SECOND / self.cps;
                                }
                            }
                        }
                    }
                });
            });

            ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                if self.is_running.load(Ordering::Relaxed) {
                    ui.label("連射中…");
                    ui.spinner();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                    if self.is_running.load(Ordering::Relaxed) {
                        if ui.button("停止").clicked() {
                            self.is_running.store(false, Ordering::Relaxed);
                        };
                        ui.add_enabled(false, egui::Button::new("開始"));
                    } else {
                        ui.add_enabled(false, egui::Button::new("停止"));
                        if ui.button("開始").clicked() {
                            self.is_running.store(true, Ordering::Relaxed);
                            tokio::spawn(auto_click(self.wait, Arc::clone(&self.is_running)));
                        };
                    }
                });
            });
        });
    }
}

async fn auto_click(wait: u64, is_running: Arc<AtomicBool>) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    while is_running.load(Ordering::Relaxed) {
        if enigo.button(Button::Left, Click).is_err() {
            std::process::exit(1);
        }
        std::thread::sleep(Duration::from_nanos(wait));
    }
}
