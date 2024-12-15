use actix_web::{web, App, HttpResponse, HttpServer, Result};
use actix_cors::Cors;
use gpt_sovits_rs::{voice_manager::VoiceManager, GPTSovits, GPTSovitsConfig};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{fs, path::Path};
use tch::Tensor;

#[derive(Debug, Serialize, Deserialize)]
struct TTSRequest {
    character: Option<String>,
    emotion: Option<String>,
    text: String,
    text_language: Option<String>,
    top_k: Option<i32>,
    top_p: Option<f32>,
    temperature: Option<f32>,
    batch_size: Option<i32>,
    speed: Option<f32>,
    save_temp: Option<bool>,
    stream: Option<bool>,
    format: Option<String>,
}
struct AppState {
    gpt_sovits: Arc<GPTSovits>,
    voice_manager: Arc<Mutex<VoiceManager>>,
}

async fn character_list(data: web::Data<AppState>) -> Result<HttpResponse> {
    let voice_manager = data.voice_manager.lock().unwrap();
    let voices = voice_manager.list_voices();
    let mut characters: Value = json!({});

    for voice in voices {
        characters[voice] = json!(["default"]);
    }

    Ok(HttpResponse::Ok().json(characters))
}

async fn tts(req: web::Query<TTSRequest>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let text_splitter = text_splitter::TextSplitter::new(50);
    let voice_manager = data.voice_manager.lock().unwrap();
    let voices = voice_manager.list_voices();

    let character = match &req.character {
        Some(c) => c.as_str(),
        None => voices
            .first()
            .ok_or_else(|| actix_web::error::ErrorInternalServerError("No voices available"))?,
    };

    let text = &req.text;

    let timer = Instant::now();

    let mut audios = vec![];

    for target_text in text_splitter.chunks(&text) {
        println!("text: {}", target_text);
        if target_text == "。" {
            continue;
        }
        let audio = data
            .gpt_sovits
            .infer(character, target_text)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        audios.push(audio);
    }

    println!("infer time: {} ms", timer.elapsed().as_millis());

    let audio = Tensor::cat(&audios, 0);
    let audio_size = audio.size1().unwrap() as usize;
    let mut samples = vec![0f32; audio_size];
    audio
        .f_copy_data(&mut samples, audio_size)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let header = wav_io::new_header(32000, 16, false, true);
    let wav_data = wav_io::write_to_bytes(&header, &samples)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("audio/wav").body(wav_data))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Initialize voice manager
    let voice_manager = Arc::new(Mutex::new(VoiceManager::new("voices")));
    if let Err(e) = voice_manager.lock().unwrap().scan_voices() {
        log::error!("Failed to scan voices directory: {}", e);
        return Ok(());
    }
    log::info!(
        "Available voices: {:?}",
        voice_manager.lock().unwrap().list_voices()
    );

    // Initialize GPT-SoVITS
    let gpt_config = GPTSovitsConfig::new("resource/ssl_model.pt".to_string()).with_chinese(
        "resource/g2pw.pt".to_string(),
        "resource/bert_model.pt".to_string(),
        "resource/tokenizer.json".to_string(),
    );

    let device = gpt_sovits_rs::Device::cuda_if_available();
    log::info!("device: {:?}", device);

    let mut gpt_sovits = gpt_config
        .build(device)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Initialize speakers
    for voice in voice_manager.lock().unwrap().list_voices() {
        let voice_ref_wav = format!("voices/{}/ref.wav", voice);
        let voice_ref_text = fs::read_to_string(Path::new(&format!("voices/{}/ref.txt", voice)))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let file = std::fs::File::open(voice_ref_wav).unwrap();
        let (head, ref_audio_samples) = wav_io::read_from_file(file).unwrap();

        gpt_sovits
            .create_speaker(
                &voice,
                &format!("voices/{}/gpt_sovits_model.pt", voice),
                &ref_audio_samples,
                head.sample_rate as usize,
                &voice_ref_text,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    }

    let gpt_sovits = Arc::new(gpt_sovits);

    let app_state = web::Data::new(AppState {
        gpt_sovits: gpt_sovits.clone(),
        voice_manager: voice_manager.clone(),
    });

    println!("Starting server at http://127.0.0.1:5000");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/character_list", web::get().to(character_list))
            .route("/tts", web::get().to(tts))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
