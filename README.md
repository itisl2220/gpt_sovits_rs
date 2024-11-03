# GPT-SoVits-rs

## Overview
This Rust project provides a library for integrating GPT-Sovits inference in Rust applications.  GPT-Sovits is a powerful tool for speech synthesis and voice conversion.  By using this library, developers can easily leverage the capabilities of GPT-Sovits within their Rust projects.

## Prerequisites
Before you can use the GPT-SoVits-rs Inference Library, you must ensure that libtorch 2.4.0 is installed on your system. libtorch is the C++ frontend for PyTorch, which is required for running the GPT-Sovits models.

You can download and install libtorch 2.4.0 from the official PyTorch website:

Go to the [PyTorch website](https://pytorch.org/).
Select the appropriate options for your system (OS, Package Manager, Python version, etc.), making sure to choose the "LibTorch" option.
Scroll down to the "Install" section and run the provided command to download and install libtorch 2.4.0.

For example, on a Linux system with CUDA support, the command might look like this:

```bash
wget https://download.pytorch.org/libtorch/cu124/libtorch-cxx11-abi-shared-with-deps-2.4.0%2Bcu124.zip
unzip libtorch-cxx11-abi-shared-with-deps-2.4.0+cu124.zip
```

After downloading and extracting the library, you may need to set environment variables to include the libtorch library path. For example:
```bash
export LIBTORCH=/path/to/libtorch
export LD_LIBRARY_PATH=$LIBTORCH/lib:$LD_LIBRARY_PATH
```
Replace /path/to/libtorch with the actual path where you extracted libtorch.

## Installation
To use this library, add the following dependency to your `Cargo.toml` file:
```toml
[dependencies]
gpt_sovits_rs = "0.0.2"
```
Replace "0.0.2" with the latest version of the library.

## Usage

```rust
use gpt_sovits_rs::GPTSovitsConfig;

fn main() {
    env_logger::init();

    let gpt_config = GPTSovitsConfig::new(
        "path/to/gpt_sovits_model.pt".to_string(),
        "path/to/ssl_model.pt".to_string(),
    ).with_cn_bert_path("path/to/bert_model.pt".to_string(), "path/to/tokenizer.json".to_string());
    // If you don't need to Chinese, you can not call `with_cn_bert_path`

    let device = gpt_sovits_rs::Device::cuda_if_available();
    log::info!("device: {:?}", device);

    let gpt_sovits = gpt_config.build(device).unwrap();

    log::info!("init done");

    let ref_text = "<Your Reference Text>";

    let ref_path = "path/to/reference_voice.wav";
    let file = std::fs::File::open(ref_path).unwrap();

    let (head, ref_audio_samples) = wav_io::read_from_file(file).unwrap();

    log::info!("load ref done");

    let text = "<What you want to say>";

    let audio = gpt_sovits.infer(
                &ref_audio_samples,
                head.sample_rate as usize,
                ref_text,
                target_text).unwrap();

    log::info!("start write file");

    let output = "out.wav";

    let audio_size = audio.size1().unwrap() as usize;
    println!("audio size: {}", audio_size);

    println!("start save audio {output}");
    let mut samples = vec![0f32; audio_size];
    audio.f_copy_data(&mut samples, audio_size).unwrap();

    println!("start write file {output}");
    let mut file_out = std::fs::File::create(output).unwrap();

    let header = wav_io::new_header(32000, 16, false, true);
    wav_io::write_to_file(&mut file_out, &header, &samples).unwrap();
    log::info!("write file done");
}

```

## Exporting GPT-Sovits Training Results
After completing the training of a GPT-Sovits model, you might need to export the training results to a .pt (PyTorch) file for use in other environments. Below are the detailed steps to export the trained model:

#### Step 1: Confirm Training Completion
Ensure that your GPT-Sovits model has finished training and that you have a trained model file available.

#### Step 2: Run the Export Script
Use the following command to run the export script and export the training results to gpt_sovits_model.pt:
```bash
python GPT_SoVITS/export_torch_script.py --gpt_model GPT_weights_v2/xxx-e15.ckpt --sovits_model SoVITS_weights_v2/xxx_e8_s248.pth --ref_audio ref.wav --ref_text 'Reference Text' --output_path output --export_common_model
```

Now you can find `gpt_sovits_model.pt`, `ssl_model.pt`, and `bert_model.pt` in the `output` directory.

The `ssl_model.pt` and `bert_model.pt` are common model files, determined by the option `--export_common_model` for whether to export, and they are not related to the trained model. Therefore, they do not need to be exported every time. 
If you do not wish to export, you can go to Hugging Face to [download](https://huggingface.co/L-jasmine/GPT_Sovits/tree/main) the resource.zip that I have already exported. And remove the `--export_common_model` when export model.