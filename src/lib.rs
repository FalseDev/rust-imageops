use image::io::Reader;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::wrap_pyfunction;
use std::io::Cursor;

pub use image::ImageFormat as ImageFormat;

type PasteLayer = (Vec<u8>, Vec<(u32, u32)>);

pub fn multi_paste(
    base: Vec<u8>,
    paste_layers: Vec<PasteLayer>,
) -> Result<image::DynamicImage, std::io::Error> {
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
    Ok(base_image)
}


pub fn paste(
    base: Vec<u8>,
    layer: Vec<u8>,
    coordinates: (u32, u32),
) -> Result<image::DynamicImage, std::io::Error> {
    multi_paste(base, vec![(layer, vec![coordinates])])
}


fn to_python_bytes(base_image: image::DynamicImage) -> PyResult<PyObject> {
    let mut output_bytes: Vec<u8> = Vec::new();
    base_image
        .write_to(&mut output_bytes, image::ImageFormat::Png)
        .expect("Output failed");

    let gil = Python::acquire_gil();
    let py = gil.python();

    let py_bytes = PyBytes::new(py, &output_bytes);

    Ok(py_bytes.to_object(py))
}


#[pymodule]
fn imageops(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "multi_paste")]
    fn multi_paste_py(base: Vec<u8>, paste_layers: Vec<PasteLayer>) -> PyResult<PyObject> {
        to_python_bytes(multi_paste(base, paste_layers)?)
    }

    #[pyfn(m, "paste")]
    fn paste_py(base: Vec<u8>, overlay: Vec<u8>, coordinates: (u32, u32)) -> PyResult<PyObject> {
        to_python_bytes(paste(base, overlay, coordinates)?)
    }

    m.add_function(wrap_pyfunction!(multi_paste_py, m)?)?;
    m.add_function(wrap_pyfunction!(paste_py, m)?)?;
    Ok(())
}
