use image::io::Reader;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::wrap_pyfunction;
use std::io::Cursor;

#[pyfunction]
fn multi_paste(base: Vec<u8>, paste_layers: Vec<(Vec<u8>, Vec<(u32, u32)>)>) -> PyResult<PyObject> {
    let mut base_image = Reader::new(Cursor::new(&base))
        .with_guessed_format()?
        .decode()
        .expect("Base image isn't decodable");

    for layer in paste_layers.iter() {
        let layer_image = Reader::new(Cursor::new(&layer.0))
            .with_guessed_format()?
            .decode()
            .expect("Base image isn't decodable");
        for coordinates in layer.1.iter() {
            image::imageops::overlay(&mut base_image, &layer_image, coordinates.0, coordinates.1);
        }
    }

    let mut output_bytes: Vec<u8> = Vec::new();
    base_image
        .write_to(&mut output_bytes, image::ImageFormat::Png)
        .expect("Output failed");

    let gil = Python::acquire_gil();
    let py = gil.python();

    let py_bytes = PyBytes::new(py, &output_bytes);

    Ok(py_bytes.to_object(py))
}

#[pyfunction]
fn paste(base: Vec<u8>, layer: Vec<u8>, coordinates: (u32, u32)) -> PyResult<PyObject> {
    multi_paste(base, vec![(layer, vec![coordinates])])
}

#[pymodule]
fn imageops(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(multi_paste, m)?)?;
    m.add_function(wrap_pyfunction!(paste, m)?)?;
    Ok(())
}
