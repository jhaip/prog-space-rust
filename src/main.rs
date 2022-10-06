use crate::database::Database;
use crate::fact::Fact;

use std::error::Error;

use std::sync::{mpsc, Arc, Mutex};
use std::{thread, time::Duration};

use opencv::{highgui, prelude::*};

pub mod database;
pub mod fact;
pub mod vision;
pub mod source_code;

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::new();
    db.claim(Fact::from_string("fox is red"));
    db.claim(Fact::from_string("rock is red"));
    db.print();
    for v in db.select(&vec!["$x is red".to_string()]) {
        println!("{:?}", v);
    }
    for v in db.select(&vec!["fox is $".to_string()]) {
        println!("{:?}", v);
    }
    for v in db.select(&vec!["%fact".to_string()]) {
        println!("{:?}", v);
    }
    db.retract("fox is $");
    db.print();

    let mut source_code_manager = source_code::SourceCodeManager::new("../../scripts/".to_string());
    source_code_manager.init(&mut db);

    let shared_frame = Arc::new(Mutex::new(Mat::default()));
    let main_frame = Arc::clone(&shared_frame);

    let (tx, rx) = mpsc::channel::<Vec<vision::SeenProgram>>();

    let vision_handle = vision::run_vision(&shared_frame, tx);

    highgui::named_window("window", highgui::WINDOW_FULLSCREEN).unwrap();
    loop {
        // TODO: handle slow rx where the video feed produces events faster than we consume them.
        // TODO: use a single_value_channel
        if let Ok(seen_programs) = rx.recv() {
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
        }
        let frame = main_frame.lock().unwrap();
        if frame.rows() > 0 {
            highgui::imshow("window", &*frame).unwrap();
        } else {
            println!("waiting for camera to give a frame")
        }
        drop(frame);
        let key = highgui::wait_key(1).unwrap();
        if key == 113 {
            // quit with q
            break;
        }
        thread::sleep(Duration::from_millis(16));
    }

    vision_handle.join().unwrap();
    Ok(())
}
