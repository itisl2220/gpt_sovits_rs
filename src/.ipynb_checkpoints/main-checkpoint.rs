use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use gpt_sovits_rs::{voice_manager::VoiceManager, GPTSovits, GPTSovitsConfig};
use hex::encode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{fs, path::Path};
use tch::Tensor;
use toml;

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

// 缓存管理结构体
struct CacheManager {
    cache_dir: String,
}

impl CacheManager {
    fn new(cache_dir: &str) -> Self {
        // 确保缓存目录存在
        if !Path::new(cache_dir).exists() {
            fs::create_dir_all(cache_dir).unwrap_or_else(|e| {
                log::warn!("无法创建缓存目录 {}: {}", cache_dir, e);
            });
        }

        CacheManager {
            cache_dir: cache_dir.to_string(),
        }
    }

    // 生成缓存文件名
    fn get_cache_filename(&self, text: &str, speaker: &str) -> String {
        let input = format!("{}{}", text, speaker);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash = encode(hasher.finalize());

        format!("{}/{}.wav", self.cache_dir, hash)
    }

    // 检查缓存是否存在
    fn cache_exists(&self, filename: &str) -> bool {
        Path::new(filename).exists()
    }

    // 从缓存加载音频
    fn load_from_cache(&self, filename: &str) -> Option<Vec<f32>> {
        if self.cache_exists(filename) {
            log::debug!("找到缓存文件: {}", filename);
            match std::fs::File::open(filename) {
                Ok(file) => match wav_io::read_from_file(file) {
                    Ok((_, samples)) => {
                        log::info!("从缓存加载音频: {}", filename);
                        return Some(samples);
                    }
                    Err(e) => {
                        log::warn!("读取缓存文件失败: {}", e);
                    }
                },
                Err(e) => {
                    log::warn!("打开缓存文件失败: {}", e);
                }
            }
        } else {
            log::debug!("缓存未命中: {}", filename);
        }
        None
    }

    // 保存音频到缓存
    fn save_to_cache(&self, filename: &str, samples: &Vec<f32>) {
        let header = wav_io::new_header(32000, 16, false, true);
        match std::fs::File::create(filename) {
            Ok(mut file) => {
                if let Err(e) = wav_io::write_to_file(&mut file, &header, samples) {
                    log::warn!("写入缓存文件失败: {}", e);
                } else {
                    log::info!("已保存音频到缓存: {}", filename);
                }
            }
            Err(e) => {
                log::warn!("创建缓存文件失败: {}", e);
            }
        }
    }

    // 清理缓存目录中的过期文件
    fn cleanup_cache(&self) {
        log::info!("开始清理缓存目录: {}", self.cache_dir);

        // 获取当前时间
        let now = std::time::SystemTime::now();

        // 读取缓存目录
        let cache_dir = Path::new(&self.cache_dir);
        if !cache_dir.exists() || !cache_dir.is_dir() {
            log::warn!("缓存目录不存在或不是目录: {}", self.cache_dir);
            return;
        }

        // 遍历缓存目录中的所有文件
        if let Ok(entries) = fs::read_dir(cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // 只处理文件
                if path.is_file() {
                    // 获取文件的最后修改时间
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            // 计算文件的年龄（以秒为单位）
                            if let Ok(duration) = now.duration_since(modified) {
                                let age_in_seconds = duration.as_secs();

                                // 如果文件超过24小时（86400秒）未修改，则删除
                                if age_in_seconds > 86400 {
                                    if let Some(filename) = path.file_name() {
                                        if let Some(filename_str) = filename.to_str() {
                                            log::info!("删除过期缓存文件: {}", filename_str);
                                            if let Err(e) = fs::remove_file(&path) {
                                                log::warn!(
                                                    "无法删除缓存文件 {}: {}",
                                                    filename_str,
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            log::warn!("无法读取缓存目录: {}", self.cache_dir);
        }

        log::info!("缓存清理完成");
    }
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

async fn tts(
    req: web::Query<TTSRequest>,
    data: web::Data<AppState>,
    cache: web::Data<Arc<Mutex<CacheManager>>>,
) -> Result<HttpResponse> {
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

    // 检查缓存
    let cache_filename = cache.lock().unwrap().get_cache_filename(text, character);
    if let Some(samples) = cache.lock().unwrap().load_from_cache(&cache_filename) {
        // 返回缓存的音频
        let header = wav_io::new_header(32000, 16, false, true);
        let wav_data = wav_io::write_to_bytes(&header, &samples)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        return Ok(HttpResponse::Ok().content_type("audio/wav").body(wav_data));
    }

    let timer = Instant::now();

    let mut audios = vec![];

    for target_text in text_splitter.chunks(&text) {
        log::info!("text: {}", target_text);
        if target_text == "。" {
            continue;
        }
        let audio = data
            .gpt_sovits
            .infer(character, target_text)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        audios.push(audio);
    }

    log::info!("infer time: {} ms", timer.elapsed().as_millis());

    let audio = Tensor::cat(&audios, 0);
    let audio_size = audio.size1().unwrap() as usize;
    let mut samples = vec![0f32; audio_size];
    audio
        .f_copy_data(&mut samples, audio_size)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // 保存到缓存
    cache
        .lock()
        .unwrap()
        .save_to_cache(&cache_filename, &samples);

    let header = wav_io::new_header(32000, 16, false, true);
    let wav_data = wav_io::write_to_bytes(&header, &samples)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().content_type("audio/wav").body(wav_data))
}

// 读取配置文件
fn read_config() -> Result<toml::Value, Box<dyn std::error::Error>> {
    let config_path = env::var("CONFIG_FILE").unwrap_or_else(|_| {
        // 尝试多个可能的位置
        let paths = vec![
            "/home/itisl/config.toml",
            "./config.toml",
            "../config.toml",
        ];

        for path in paths {
            if Path::new(path).exists() {
                return path.to_string();
            }
        }

        // 如果都不存在，返回默认路径
        "/home/itisl/config.toml".to_string()
    });

    let config_str = fs::read_to_string(config_path)?;
    let config: toml::Value = toml::from_str(&config_str)?;
    Ok(config)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志系统
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("GPT-SoVITS 服务启动中...");

    // 尝试读取配置文件
    let config = match read_config() {
        Ok(config) => {
            log::info!("成功加载配置文件");
            config
        }
        Err(e) => {
            log::warn!("无法读取配置文件: {}, 将使用默认配置", e);
            toml::Value::Table(toml::map::Map::new())
        }
    };

    // 从配置或环境变量获取缓存目录
    let cache_dir = match config.get("cache_dir") {
        Some(toml::Value::String(dir)) => dir.clone(),
        _ => {
            env::var("GPT_SOVITS_CACHE_DIR").unwrap_or_else(|_| "/home/itisl/tmp".to_string())
        }
    };
    let cache_manager = Arc::new(Mutex::new(CacheManager::new(&cache_dir)));

    // 每隔2小时清理一次缓存
    let cleanup_cache_manager = cache_manager.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(7200)).await;
            cleanup_cache_manager.lock().unwrap().cleanup_cache();
        }
    });

    // 从命令行参数获取端口号，默认为6006
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse::<u16>().unwrap_or(6006)
    } else {
        6006
    };

    // Initialize voice manager
    let voice_manager = Arc::new(Mutex::new(VoiceManager::new("voices")));
    if let Err(e) = voice_manager.lock().unwrap().scan_voices() {
        log::error!("Failed to scan voices directory: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
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

    log::info!("Starting server at http://127.0.0.1:{}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .app_data(web::Data::new(cache_manager.clone()))
            .route("/character_list", web::get().to(character_list))
            .route("/tts", web::get().to(tts))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
