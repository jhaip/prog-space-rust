use crate::database::Database;
use crate::fact::Fact;

use std::error::Error;

use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread, time::Duration};

use nannou::wgpu::Texture;
use opencv::{highgui, prelude::*};

use std::time::Instant;

use nannou::prelude::*;

pub mod database;
pub mod fact;
pub mod illumination;
pub mod source_code;
pub mod vision;

use lazy_static::lazy_static;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{QueryResult, QueryResultVariable};
    use crate::fact::Term;

    #[test]
    fn exploration() {
        let mut db = Database::new();
        db.claim(Fact::from_string("fox is red"));
        db.claim(Fact::from_string("rock is red"));
        db.print();
        let q1 = db.select(&vec!["$x is red".to_string()]).iter();
        assert_eq!(
            q1.next(),
            QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "x".to_string(),
                    term: Term::String("fox")
                }]
            }
        );
        assert_eq!(
            q1.next(),
            QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "x".to_string(),
                    term: Term::String("rock")
                }]
            }
        );

        let q2 = db.select(&vec!["fox is $".to_string()]).iter();
        assert_eq!(q2.next(), QueryResult { result: vec![] });

        let q3 = db.select(&vec!["%fact".to_string()]).iter();
        assert_eq!(
            q3.next(),
            QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "fact".to_string(),
                    term: Term::String("fox is red")
                }]
            }
        );
        assert_eq!(
            q3.next(),
            QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "fact".to_string(),
                    term: Term::String("rock is red")
                }]
            }
        );

        db.retract("fox is $");

        let q4 = db.select(&vec!["$x is red".to_string()]).iter();
        assert_eq!(
            q4.next(),
            QueryResult {
                result: vec![QueryResultVariable {
                    variable_name: "x".to_string(),
                    term: Term::String("rock")
                }]
            }
        );
    }
}

struct Model {
    vision_handle: JoinHandle<()>,
    static_db: &'static Mutex<Database>,
    main_frame: Arc<Mutex<Mat>>,
    rx: mpsc::Receiver<Vec<crate::vision::SeenProgram>>,
}

fn main() {
    nannou::app(model)
        .update(update)
        .exit(exit)
        .simple_window(view)
        .run();
}

fn exit(app: &App, model: Model) {
    model.vision_handle.join().unwrap();
}

fn model(_app: &App) -> Model {
    lazy_static! {
        static ref static_db: Mutex<Database> = Mutex::new(Database::new());
    }

    let mut source_code_manager = source_code::SourceCodeManager::new("../../scripts/".to_string());
    // source_code_manager.init(&mut db);
    source_code_manager.init(&static_db);
    let start = Instant::now();
    source_code_manager.update(&static_db);
    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
    static_db.lock().unwrap().print();

    let shared_frame = Arc::new(Mutex::new(Mat::default()));
    let main_frame = Arc::clone(&shared_frame);

    let (tx, rx) = mpsc::channel::<Vec<vision::SeenProgram>>();

    Model {
        vision_handle: vision::run_vision(&shared_frame, tx),
        static_db: &static_db,
        main_frame: main_frame,
        rx: rx,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    println!("FPS: {}", _app.fps());
}

fn view(_app: &App, _model: &Model, _frame: Frame) {
    _frame.clear(BLACK);
    let draw = _app.draw();

    // TODO: handle slow rx where the video feed produces events faster than we consume them.
    // TODO: use a single_value_channel
    let mut db = _model.static_db.lock().unwrap();
    if let Ok(seen_programs) = _model.rx.recv() {
        // println!("---{:?}", seen_programs);
        db.retract("#0cv %");
        for p in seen_programs.iter() {
            db.claim(Fact::from_string(
                format!(
                    "#0cv program {} at {} {} {} {} {} {} {} {}",
                    p.id,
                    p.corner1.x,
                    p.corner1.y,
                    p.corner2.x,
                    p.corner2.y,
                    p.corner3.x,
                    p.corner3.y,
                    p.corner4.x,
                    p.corner4.y
                )
                .as_str(),
            ));
        }

        let frame = _model.main_frame.lock().unwrap();
        if frame.rows() > 0 {
            // TODO
            // highgui::imshow("window", &*frame).unwrap();
            // let buffer = *frame; // TODO: convert
            if frame.is_continuous() {
                let data_bytes: &[u8] = frame.data_bytes().unwrap();
                let size = frame.size().unwrap();
                let tex = Texture::from_image(
                    _app,
                    &nannou::image::DynamicImage::ImageBgr8(nannou::image::ImageBuffer::from_raw(
                        size.width.to_u32().unwrap(),
                        size.height.to_u32().unwrap(),
                        data_bytes.to_vec()
                    ).unwrap()),
                );
                draw.texture(&tex);
            }
        } else {
            println!("waiting for camera to give a frame")
        }
        drop(frame);
    }

    draw.to_frame(_app, &_frame).unwrap();
}
