pub use opencv::core::Point2f;
use opencv::{
    aruco,
    prelude::*,
    types::{VectorOfVectorOfPoint2f, VectorOfi32},
    videoio,
};
use std::sync::{mpsc, Arc, Mutex};
use std::{thread, time::Duration};

#[derive(Debug)]
pub struct SeenProgram {
    pub id: i32,
    pub corner1: Point2f,
    pub corner2: Point2f,
    pub corner3: Point2f,
    pub corner4: Point2f,
}

pub fn run_vision(
    shared_frame: &Arc<Mutex<Mat>>,
    tx: mpsc::Sender<Vec<SeenProgram>>,
) -> thread::JoinHandle<()> {
    let cv_frame = Arc::clone(&shared_frame);
    thread::spawn(move || {
        // Open the web-camera (assuming you have one)
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
        // let mut frame = Mat::default(); // This array will store the web-cam data
        let dictionary =
            aruco::get_predefined_dictionary(aruco::PREDEFINED_DICTIONARY_NAME::DICT_6X6_1000)
                .unwrap();
        let mut corners = VectorOfVectorOfPoint2f::default();
        let mut ids = VectorOfi32::default();
        let detector_parameters = aruco::DetectorParameters::default().unwrap();
        let detector_parameters_ptr = opencv::core::Ptr::new(detector_parameters);
        let mut rejected_img_points = VectorOfVectorOfPoint2f::default();
        loop {
            let mut frame = cv_frame.lock().unwrap();
            cam.read(&mut *frame).unwrap();
            aruco::detect_markers(
                &*frame,
                &dictionary,
                &mut corners,
                &mut ids,
                &detector_parameters_ptr,
                &mut rejected_img_points,
            )
            .unwrap();
            aruco::draw_detected_markers(
                &mut *frame,
                &corners,
                &ids,
                opencv::core::VecN([0., 0., 255., 255.]),
            )
            .unwrap();
            drop(frame);
            let seen_programs: Vec<SeenProgram> = ids
                .iter()
                .zip(corners.iter())
                .map(|(_id, _corners)| SeenProgram {
                    id: _id,
                    corner1: _corners.get(0).unwrap(),
                    corner2: _corners.get(1).unwrap(),
                    corner3: _corners.get(2).unwrap(),
                    corner4: _corners.get(3).unwrap(),
                })
                .collect();
            tx.send(seen_programs).unwrap();
            // println!("{:?}", ids);
            thread::sleep(Duration::from_millis(16));
        }
    })
}
