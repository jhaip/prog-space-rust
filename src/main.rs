use crate::database::Database;
use crate::fact::Fact;

use std::error::Error;

use opencv::{
    aruco,
    highgui,
    prelude::*,
    types::{VectorOfVectorOfPoint2f, VectorOfi32},
    videoio,
};

pub mod database;
pub mod fact;

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

    highgui::named_window("window", highgui::WINDOW_FULLSCREEN)?;
    // Open the web-camera (assuming you have one)
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    let mut frame = Mat::default(); // This array will store the web-cam data
    let mut dictionary =
        aruco::get_predefined_dictionary(aruco::PREDEFINED_DICTIONARY_NAME::DICT_6X6_1000)?;
    let mut corners = VectorOfVectorOfPoint2f::default();
    let mut ids = VectorOfi32::default();
    let detector_parameters = aruco::DetectorParameters::default()?;
    let detector_parameters_ptr = opencv::core::Ptr::new(detector_parameters);
    let mut rejected_img_points = VectorOfVectorOfPoint2f::default();
    // let mut dst_img = opencv::core::Mat::default();
    // Read the camera
    // and display in the window
    loop {
        cam.read(&mut frame)?;
        aruco::detect_markers(
            &frame,
            &dictionary,
            &mut corners,
            &mut ids,
            &detector_parameters_ptr,
            &mut rejected_img_points,
        )?;
        aruco::draw_detected_markers(
            &mut frame,
            &corners,
            &ids,
            opencv::core::VecN([0., 0., 255., 255.]),
        )?;
        println!("{:?}", ids);
        highgui::imshow("window", &frame)?;
        let key = highgui::wait_key(1)?;
        if key == 113 {
            // quit with q
            break;
        }
    }
    Ok(())
}
