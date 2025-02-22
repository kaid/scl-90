use eframe::egui;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
struct Question {
    #[serde(rename = "#")]
    number: String,
    #[serde(rename = "Question Text")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct ScaleConfig {
    name: String,
    questions: Vec<usize>,
}

#[derive(Debug)]
struct Scale {
    name: String,
    questions: Vec<usize>,
    score: f32,
    items_answered: usize,
}

struct Scl90App {
    questions: Vec<Question>,
    answers: Vec<Option<i32>>,
    total_score: i32,
    scales: HashMap<String, Scale>,
    answered_count: usize,
    import_text: String,
}

impl Scl90App {
    fn new() -> Self {
        // Load questions from CSV
        let file = File::open("assets/scl-90.csv").expect("Failed to open questions file");
        let mut rdr = csv::Reader::from_reader(file);
        let questions: Vec<Question> = rdr.deserialize().filter_map(|result| result.ok()).collect();

        let answers = vec![None; questions.len()];

        // Load scales from RON file
        let mut scales_file = File::open("assets/scales.ron").expect("Failed to open scales file");
        let mut scales_content = String::new();
        scales_file
            .read_to_string(&mut scales_content)
            .expect("Failed to read scales file");

        let scales_config: HashMap<String, ScaleConfig> =
            ron::from_str(&scales_content).expect("Failed to parse scales configuration");

        // Convert ScaleConfig to Scale
        let scales = scales_config
            .into_iter()
            .map(|(key, config)| {
                (
                    key,
                    Scale {
                        name: config.name,
                        questions: config.questions,
                        score: 0.0,
                        items_answered: 0,
                    },
                )
            })
            .collect();

        Self {
            questions,
            answers,
            total_score: 0,
            scales,
            answered_count: 0,
            import_text: String::new(),
        }
    }

    fn calculate_scores(&mut self) {
        self.answered_count = self.answers.iter().filter(|x| x.is_some()).count();
        // Calculate total score
        self.total_score = self.answers.iter().filter_map(|&x| x).sum();

        // Calculate scores for each scale
        for scale in self.scales.values_mut() {
            let mut scale_sum = 0;
            scale.items_answered = 0;

            for &q_num in &scale.questions {
                if let Some(score) = self.answers[q_num - 1] {
                    scale_sum += score;
                    scale.items_answered += 1;
                }
            }

            scale.score = if scale.items_answered > 0 {
                scale_sum as f32 / scale.items_answered as f32
            } else {
                0.0
            };
        }
    }

    fn answers_to_string(&self) -> String {
        self.answers
            .iter()
            .map(|opt| match opt {
                Some(n) => n.to_string(),
                None => "-".to_string(),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn load_answers_from_string(&mut self, s: &str) -> Result<(), String> {
        if s.len() != self.questions.len() {
            return Err("Invalid answer string length".to_string());
        }

        self.answers = s
            .chars()
            .map(|c| match c {
                '-' => None,
                '0'..='4' => c.to_digit(10).map(|n| n as i32),
                _ => None,
            })
            .collect();
        self.calculate_scores();
        Ok(())
    }

    fn get_severity_level(score: f32) -> (&'static str, egui::Color32) {
        match score {
            s if s < 0.5 => ("Normal", egui::Color32::GREEN),
            s if s < 1.0 => ("Mild", egui::Color32::YELLOW),
            s if s < 2.0 => ("Moderate", egui::Color32::GOLD),
            s if s < 3.0 => ("Severe", egui::Color32::RED),
            _ => ("Extreme", egui::Color32::DARK_RED),
        }
    }
}

impl eframe::App for Scl90App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("SCL-90 Questionnaire");

                // Add import/export UI
                ui.horizontal(|ui| {
                    if ui.button("Export Answers").clicked() {
                        ui.ctx().copy_text(self.answers_to_string());
                    }

                    ui.text_edit_singleline(&mut self.import_text);
                    if ui.button("Import Answers").clicked() {
                        // Clone the text before using it
                        let import_text = self.import_text.clone();
                        if let Err(e) = self.load_answers_from_string(&import_text) {
                            eprintln!("Error loading answers: {}", e);
                        }
                    }

                    if ui.button("Reset").clicked() {
                        self.answers = vec![None; self.questions.len()];
                        self.calculate_scores();
                    }
                });

                // Progress indicator
                ui.add_space(10.0);
                ui.label(format!("Questions answered: {}/90", self.answered_count));
                let mut changed_answer = false;
                let mut changed_index = 0;
                let mut new_value = None;
                ui.add_space(10.0);

                for (index, question) in self.questions.iter().enumerate() {
                    ui.vertical(|ui| {
                        ui.label(format!("{}. {}:", question.number, question.text));
                        ui.add_space(5.0);
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
                    ui.add_space(10.0);
                }

                if changed_answer {
                    self.answers[changed_index] = new_value;
                    self.calculate_scores();
                }

                ui.separator();

                // Display results
                ui.heading("Results");
                if self.answered_count > 0 {
                    let gsi = self.total_score as f32 / self.answered_count as f32;
                    let (severity, color) = Self::get_severity_level(gsi);
                    ui.horizontal(|ui| {
                        ui.label("Global Severity Index (GSI):");
                        ui.colored_label(color, format!("{:.2} - {}", gsi, severity));
                    });

                    ui.separator();
                    ui.heading("Scale Scores:");

                    for scale in self.scales.values() {
                        if scale.items_answered > 0 {
                            let (severity, color) = Self::get_severity_level(scale.score);
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: ", scale.name));
                                ui.colored_label(
                                    color,
                                    format!(
                                        "{:.2} - {} ({}/{} items answered)",
                                        scale.score,
                                        severity,
                                        scale.items_answered,
                                        scale.questions.len()
                                    ),
                                );
                            });
                        }
                    }
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SCL-90 Questionnaire",
        native_options,
        Box::new(|_cc| Ok(Box::new(Scl90App::new()))),
    )
}
