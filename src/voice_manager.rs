use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct VoiceModel {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct VoiceManager {
    voices_dir: PathBuf,
    voices: HashMap<String, VoiceModel>,
}

impl VoiceManager {
    pub fn new<P: AsRef<Path>>(voices_dir: P) -> Self {
        Self {
            voices_dir: voices_dir.as_ref().to_path_buf(),
            voices: HashMap::new(),
        }
    }

    pub fn scan_voices(&mut self) -> std::io::Result<()> {
        self.voices.clear();
        
        // Create voices directory if it doesn't exist
        if !self.voices_dir.exists() {
            fs::create_dir_all(&self.voices_dir)?;
        }

        // Scan for voice model directories
        for entry in fs::read_dir(&self.voices_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                
                // Check if this directory contains required model files
                // TODO: Add specific file checks based on your model requirements
                
                let voice_model = VoiceModel {
                    name: name.clone(),
                    path,
                };
                
                self.voices.insert(name, voice_model);
            }
        }

        Ok(())
    }

    pub fn get_voice(&self, name: &str) -> Option<&VoiceModel> {
        self.voices.get(name)
    }

    pub fn list_voices(&self) -> Vec<&str> {
        self.voices.keys().map(|s| s.as_str()).collect()
    }
}
