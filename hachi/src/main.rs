use ndarray::Array2;
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::TensorRef,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file("models/big_model.onnx")?;

    let input_data = Array2::from_shape_vec((1, 1650), vec![0.0f32; 1650])?;

    let input_ref = TensorRef::from_array_view((input_data.shape(), input_data.as_slice().unwrap()))?;

    let outputs = session.run(ort::inputs!["input" => input_ref])?;

    let output_tensor = outputs["variable"].try_extract_tensor::<f32>()?;
    println!("Prediction: {:?}", output_tensor);

    Ok(())
}