use eframe::egui;
use serde::Deserialize;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Question {
    #[serde(rename = "#")]
    number: String,
    #[serde(rename = "Question Text")]
    text: String,
}

struct Scl90App {
    questions: Vec<Question>,
    answers: Vec<Option<i32>>,
    total_score: i32,
}

impl Scl90App {
    fn new() -> Self {
        let file = File::open("assets/scl-90.csv").expect("Failed to open file");
        let mut rdr = csv::Reader::from_reader(file);
        let questions: Vec<Question> = rdr
            .deserialize()
            .filter_map(|result| result.ok())
            .collect();
        
        let answers = vec![None; questions.len()];
        
        Self {
            questions,
            answers,
            total_score: 0,
        }
    }

    fn calculate_total(&mut self) {
        self.total_score = self.answers.iter()
            .filter_map(|&x| x)
            .sum();
    }
}

impl eframe::App for Scl90App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("SCL-90 Questionnaire");
                
                let mut changed_answer = false;
                let mut changed_index = 0;
                let mut new_value = None;
                
                for (index, question) in self.questions.iter().enumerate() {
                    ui.group(|ui| {
                        ui.label(format!("{}. {}", question.number, question.text));
                        ui.horizontal(|ui| {
                            for score in 0..=4 {
                                let text = match score {
                                    0 => "Not at All",
                                    1 => "A Little Bit",
                                    2 => "Moderately",
                                    3 => "Quite a Bit",
                                    4 => "Extremely",
                                    _ => unreachable!(),
    };
    
                                let selected = self.answers[index] == Some(score);
                                if ui.radio(selected, text).clicked() {
                                    changed_answer = true;
                                    changed_index = index;
                                    new_value = Some(score);
                                }
                            }
                        });
                    });
                }
                
                if changed_answer {
                    self.answers[changed_index] = new_value;
                    self.calculate_total();
                }
                
                ui.separator();
                ui.heading(format!("Total Score: {}", self.total_score));
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "SCL-90 Questionnaire",
        native_options,
        Box::new(|_cc| Box::new(Scl90App::new()))
    )
}