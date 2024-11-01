use gpt_sovits_rs::GPTSovitsConfig;
use std::sync::Arc;
use std::thread;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let gpt_config = GPTSovitsConfig::new(
        "resource/weights/hejiawen/gpt_sovits_model.pt".to_string(),
        "resource/weights/hejiawen/ssl_model.pt".to_string(),
    );

    let device = gpt_sovits_rs::Device::cuda_if_available();
    println!("device: {:?}", device);

    let gpt_sovits = gpt_config.build(device)?;

    log::info!("init done");

    let ref_text = "刚刚打开直播间，欢迎所有走过路过直直播间的家人们啊";

    let ref_path =
        "resource/weights/hejiawen/刚刚打开直播间，欢迎所有走过路过直直播间的家人们啊.wav";
    let file = std::fs::File::open(ref_path).unwrap();

    let (head, ref_audio_samples) = wav_io::read_from_file(file).unwrap();

    log::info!("load ref done");
    let audio_samples = Arc::new(ref_audio_samples);
    let gpt_sovits = Arc::new(gpt_sovits);

    let target_texts = vec![
        "每一款产品都经过严格的质量控制，确保客户的满意度。",
        // "在设计上，我们追求简约与时尚的完美结合，吸引年轻用户的关注。",
        // "通过不断的技术创新，我们力求引领行业发展潮流。",
    ];

    let handles: Vec<_> = target_texts
        .into_iter()
        .map(|target_text| {
            let gpt_sovits = Arc::clone(&gpt_sovits);
            let audio_samples = Arc::clone(&audio_samples);

            thread::spawn(move || {
                let time = std::time::Instant::now();
                let ref_audio_samples = audio_samples.clone();
                let audio = match gpt_sovits.infer(
                    &ref_audio_samples,
                    head.sample_rate as usize,
                    ref_text,
                    target_text,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("error: {}", e);
                        return;
                    }
                };
                log::info!("start write file");
                println!("time: {:?}", time.elapsed());

                let output = format!("out_{}.wav", target_text);

                let audio_size = audio.size1().unwrap() as usize;
                println!("audio size: {}", audio_size);

                println!("start save audio {}", output);
                let mut samples = vec![0f32; audio_size];
                audio.f_copy_data(&mut samples, audio_size).unwrap();

                println!("start write file {}", output);
                let mut file_out = std::fs::File::create(&output).unwrap();
                let header = wav_io::new_header(32000, 16, false, true);
                wav_io::write_to_file(&mut file_out, &header, &samples).unwrap();
                log::info!("write file done");
            })
        })
        .collect();

    let time = std::time::Instant::now();
    for handle in handles {
        handle.join().unwrap();
    }

    println!("time: {:?}", time.elapsed());
    Ok(())
}
