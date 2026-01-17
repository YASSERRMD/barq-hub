//! Voice (TTS/STT) API handlers

use axum::{
    extract::{Path, State, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::api::AppState;

// ============================================================================
// TTS Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct TTSProvider {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub description: Option<String>,
    pub supports_streaming: bool,
    pub supports_realtime: bool,
    pub supports_custom_voices: bool,
    pub supported_languages: Vec<String>,
    pub price_per_million_chars: f64,
    pub enabled: bool,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TTSVoice {
    pub id: String,
    pub provider_id: String,
    pub voice_id: String,
    pub name: String,
    pub language: String,
    pub gender: Option<String>,
    pub style: Option<String>,
    pub quality_tier: String,
    pub preview_url: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct TTSRequest {
    pub text: String,
    pub voice_id: String,
    #[serde(default = "default_format")]
    pub format: String, // mp3, wav, ogg
    #[serde(default)]
    pub speed: Option<f32>,
    #[serde(default)]
    pub pitch: Option<f32>,
}

fn default_format() -> String {
    "mp3".to_string()
}

#[derive(Debug, Serialize)]
pub struct TTSResponse {
    pub audio_url: Option<String>,
    pub audio_base64: Option<String>,
    pub format: String,
    pub duration_seconds: f64,
    pub characters_processed: i32,
    pub provider: String,
    pub voice: String,
}

// ============================================================================
// STT Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct STTProvider {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub description: Option<String>,
    pub supports_streaming: bool,
    pub supports_realtime: bool,
    pub supports_diarization: bool,
    pub supported_languages: Vec<String>,
    pub word_error_rate: Option<f64>,
    pub typical_latency_ms: Option<i32>,
    pub price_per_hour: f64,
    pub enabled: bool,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct STTModel {
    pub id: String,
    pub provider_id: String,
    pub model_id: String,
    pub name: String,
    pub languages: Vec<String>,
    pub word_error_rate: Option<f64>,
    pub latency_ms: Option<i32>,
    pub supports_streaming: bool,
    pub supports_diarization: bool,
    pub quality_tier: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct STTRequest {
    pub audio_base64: Option<String>,
    pub audio_url: Option<String>,
    #[serde(default = "default_stt_model")]
    pub model: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub diarization: bool,
    #[serde(default)]
    pub punctuation: bool,
}

fn default_stt_model() -> String {
    "nova-3".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

#[derive(Debug, Serialize)]
pub struct STTResponse {
    pub text: String,
    pub confidence: f64,
    pub words: Vec<WordTiming>,
    pub duration_seconds: f64,
    pub language: String,
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speakers: Option<Vec<Speaker>>,
}

#[derive(Debug, Serialize)]
pub struct WordTiming {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct Speaker {
    pub id: i32,
    pub segments: Vec<SpeakerSegment>,
}

#[derive(Debug, Serialize)]
pub struct SpeakerSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

// ============================================================================
// WebRTC Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateWebRTCSession {
    pub session_type: String, // voice_chat, transcription, tts_preview
    pub stt_provider_id: Option<String>,
    pub tts_provider_id: Option<String>,
    pub stt_model_id: Option<String>,
    pub tts_voice_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WebRTCSession {
    pub id: String,
    pub session_type: String,
    pub status: String,
    pub ice_servers: Vec<IceServer>,
    pub stt_provider: Option<String>,
    pub tts_provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

// ============================================================================
// TTS Handlers
// ============================================================================

/// List all TTS providers
pub async fn list_tts_providers(
    State(_state): State<Arc<AppState>>,
) -> Json<Vec<TTSProvider>> {
    // Return configured TTS providers
    let providers = vec![
        TTSProvider {
            id: "tts-elevenlabs".to_string(),
            name: "ElevenLabs".to_string(),
            provider_type: "elevenlabs".to_string(),
            description: Some("Most realistic voices, 1000+ options, emotional prosody".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_custom_voices: true,
            supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "ar".to_string(), "zh".to_string()],
            price_per_million_chars: 30.0,
            enabled: true,
            is_default: true,
        },
        TTSProvider {
            id: "tts-openai".to_string(),
            name: "OpenAI TTS".to_string(),
            provider_type: "openai".to_string(),
            description: Some("Natural conversational flow, multilingual excellence".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_custom_voices: false,
            supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "ja".to_string(), "ko".to_string()],
            price_per_million_chars: 15.0,
            enabled: true,
            is_default: false,
        },
        TTSProvider {
            id: "tts-google".to_string(),
            name: "Google Cloud TTS".to_string(),
            provider_type: "google".to_string(),
            description: Some("Broadcast-quality, 125+ languages, WaveNet/Neural2".to_string()),
            supports_streaming: true,
            supports_realtime: false,
            supports_custom_voices: true,
            supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "it".to_string(), "pt".to_string()],
            price_per_million_chars: 16.0,
            enabled: true,
            is_default: false,
        },
        TTSProvider {
            id: "tts-minimax".to_string(),
            name: "MiniMax Talkie".to_string(),
            provider_type: "minimax".to_string(),
            description: Some("Ultra-low latency, Mandarin/Arabic leader".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_custom_voices: false,
            supported_languages: vec!["zh".to_string(), "ar".to_string(), "en".to_string()],
            price_per_million_chars: 8.0,
            enabled: true,
            is_default: false,
        },
        TTSProvider {
            id: "tts-azure".to_string(),
            name: "Azure Neural TTS".to_string(),
            provider_type: "azure".to_string(),
            description: Some("Enterprise-grade consistency, custom voices".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_custom_voices: true,
            supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "zh".to_string()],
            price_per_million_chars: 16.0,
            enabled: true,
            is_default: false,
        },
    ];
    
    Json(providers)
}

/// List voices for a TTS provider
pub async fn list_tts_voices(
    State(_state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Json<Vec<TTSVoice>> {
    // Sample voices per provider
    let voices: Vec<TTSVoice> = match provider_id.as_str() {
        "tts-elevenlabs" => vec![
            TTSVoice { id: "voice-el-rachel".to_string(), provider_id: provider_id.clone(), voice_id: "rachel".to_string(), name: "Rachel".to_string(), language: "en-US".to_string(), gender: Some("female".to_string()), style: Some("conversational".to_string()), quality_tier: "premium".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-el-josh".to_string(), provider_id: provider_id.clone(), voice_id: "josh".to_string(), name: "Josh".to_string(), language: "en-US".to_string(), gender: Some("male".to_string()), style: Some("narrative".to_string()), quality_tier: "premium".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-el-bella".to_string(), provider_id: provider_id.clone(), voice_id: "bella".to_string(), name: "Bella".to_string(), language: "en-GB".to_string(), gender: Some("female".to_string()), style: Some("conversational".to_string()), quality_tier: "premium".to_string(), preview_url: None, enabled: true },
        ],
        "tts-openai" => vec![
            TTSVoice { id: "voice-oai-alloy".to_string(), provider_id: provider_id.clone(), voice_id: "alloy".to_string(), name: "Alloy".to_string(), language: "en-US".to_string(), gender: Some("neutral".to_string()), style: Some("conversational".to_string()), quality_tier: "neural".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-oai-echo".to_string(), provider_id: provider_id.clone(), voice_id: "echo".to_string(), name: "Echo".to_string(), language: "en-US".to_string(), gender: Some("male".to_string()), style: Some("conversational".to_string()), quality_tier: "neural".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-oai-nova".to_string(), provider_id: provider_id.clone(), voice_id: "nova".to_string(), name: "Nova".to_string(), language: "en-US".to_string(), gender: Some("female".to_string()), style: Some("conversational".to_string()), quality_tier: "neural".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-oai-shimmer".to_string(), provider_id: provider_id.clone(), voice_id: "shimmer".to_string(), name: "Shimmer".to_string(), language: "en-US".to_string(), gender: Some("female".to_string()), style: Some("soft".to_string()), quality_tier: "neural".to_string(), preview_url: None, enabled: true },
        ],
        "tts-google" => vec![
            TTSVoice { id: "voice-gc-wavenet-a".to_string(), provider_id: provider_id.clone(), voice_id: "en-US-Wavenet-A".to_string(), name: "WaveNet A".to_string(), language: "en-US".to_string(), gender: Some("male".to_string()), style: Some("standard".to_string()), quality_tier: "neural".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-gc-wavenet-c".to_string(), provider_id: provider_id.clone(), voice_id: "en-US-Wavenet-C".to_string(), name: "WaveNet C".to_string(), language: "en-US".to_string(), gender: Some("female".to_string()), style: Some("standard".to_string()), quality_tier: "neural".to_string(), preview_url: None, enabled: true },
        ],
        "tts-minimax" => vec![
            TTSVoice { id: "voice-mm-alice".to_string(), provider_id: provider_id.clone(), voice_id: "alice".to_string(), name: "Alice".to_string(), language: "en-US".to_string(), gender: Some("female".to_string()), style: Some("conversational".to_string()), quality_tier: "premium".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-mm-xiaoming".to_string(), provider_id: provider_id.clone(), voice_id: "xiaoming".to_string(), name: "Xiaoming".to_string(), language: "zh-CN".to_string(), gender: Some("male".to_string()), style: Some("conversational".to_string()), quality_tier: "premium".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-mm-fatima".to_string(), provider_id: provider_id.clone(), voice_id: "fatima".to_string(), name: "Fatima".to_string(), language: "ar-SA".to_string(), gender: Some("female".to_string()), style: Some("conversational".to_string()), quality_tier: "premium".to_string(), preview_url: None, enabled: true },
        ],
        "tts-azure" => vec![
            TTSVoice { id: "voice-az-jenny".to_string(), provider_id: provider_id.clone(), voice_id: "en-US-JennyNeural".to_string(), name: "Jenny".to_string(), language: "en-US".to_string(), gender: Some("female".to_string()), style: Some("conversational".to_string()), quality_tier: "neural".to_string(), preview_url: None, enabled: true },
            TTSVoice { id: "voice-az-guy".to_string(), provider_id: provider_id.clone(), voice_id: "en-US-GuyNeural".to_string(), name: "Guy".to_string(), language: "en-US".to_string(), gender: Some("male".to_string()), style: Some("news".to_string()), quality_tier: "neural".to_string(), preview_url: None, enabled: true },
        ],
        _ => vec![],
    };
    
    Json(voices)
}

/// Synthesize speech from text
pub async fn synthesize_speech(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<TTSRequest>,
) -> Json<TTSResponse> {
    // Demo response - in production, call actual TTS provider
    Json(TTSResponse {
        audio_url: Some(format!("https://api.synapse.local/v1/audio/tts_{}.mp3", uuid::Uuid::new_v4())),
        audio_base64: None,
        format: request.format,
        duration_seconds: (request.text.len() as f64) / 15.0, // Rough estimate
        characters_processed: request.text.len() as i32,
        provider: "elevenlabs".to_string(),
        voice: request.voice_id,
    })
}

// ============================================================================
// STT Handlers
// ============================================================================

/// List all STT providers
pub async fn list_stt_providers(
    State(_state): State<Arc<AppState>>,
) -> Json<Vec<STTProvider>> {
    let providers = vec![
        STTProvider {
            id: "stt-deepgram".to_string(),
            name: "Deepgram Nova-3".to_string(),
            provider_type: "deepgram".to_string(),
            description: Some("11-14% WER, <300ms real-time streaming".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_diarization: true,
            supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "zh".to_string(), "ar".to_string()],
            word_error_rate: Some(12.5),
            typical_latency_ms: Some(280),
            price_per_hour: 0.25,
            enabled: true,
            is_default: true,
        },
        STTProvider {
            id: "stt-openai".to_string(),
            name: "OpenAI Whisper v3".to_string(),
            provider_type: "openai".to_string(),
            description: Some("11.6% WER, 100+ languages, noise robust".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_diarization: false,
            supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "ja".to_string(), "ko".to_string(), "ar".to_string(), "hi".to_string()],
            word_error_rate: Some(11.6),
            typical_latency_ms: Some(500),
            price_per_hour: 0.006,
            enabled: true,
            is_default: false,
        },
        STTProvider {
            id: "stt-assemblyai".to_string(),
            name: "AssemblyAI Universal-2".to_string(),
            provider_type: "assemblyai".to_string(),
            description: Some("14.5% WER + diarization/summaries".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_diarization: true,
            supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string()],
            word_error_rate: Some(14.5),
            typical_latency_ms: Some(400),
            price_per_hour: 0.37,
            enabled: true,
            is_default: false,
        },
        STTProvider {
            id: "stt-google".to_string(),
            name: "Google Cloud Chirp".to_string(),
            provider_type: "google".to_string(),
            description: Some("11.6% WER, enterprise scale".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_diarization: true,
            supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "zh".to_string(), "ja".to_string()],
            word_error_rate: Some(11.6),
            typical_latency_ms: Some(350),
            price_per_hour: 0.024,
            enabled: true,
            is_default: false,
        },
        STTProvider {
            id: "stt-minimax".to_string(),
            name: "MiniMax STT".to_string(),
            provider_type: "minimax".to_string(),
            description: Some("Mandarin/Arabic specialist, cost-effective streaming".to_string()),
            supports_streaming: true,
            supports_realtime: true,
            supports_diarization: false,
            supported_languages: vec!["zh".to_string(), "ar".to_string(), "en".to_string()],
            word_error_rate: Some(10.0),
            typical_latency_ms: Some(200),
            price_per_hour: 0.15,
            enabled: true,
            is_default: false,
        },
    ];
    
    Json(providers)
}

/// List models for an STT provider
pub async fn list_stt_models(
    State(_state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Json<Vec<STTModel>> {
    let models: Vec<STTModel> = match provider_id.as_str() {
        "stt-deepgram" => vec![
            STTModel { id: "model-dg-nova3".to_string(), provider_id: provider_id.clone(), model_id: "nova-3".to_string(), name: "Nova 3".to_string(), languages: vec!["en".to_string(), "es".to_string(), "fr".to_string()], word_error_rate: Some(11.0), latency_ms: Some(280), supports_streaming: true, supports_diarization: true, quality_tier: "premium".to_string(), enabled: true },
            STTModel { id: "model-dg-nova2".to_string(), provider_id: provider_id.clone(), model_id: "nova-2".to_string(), name: "Nova 2".to_string(), languages: vec!["en".to_string(), "es".to_string()], word_error_rate: Some(14.0), latency_ms: Some(300), supports_streaming: true, supports_diarization: true, quality_tier: "standard".to_string(), enabled: true },
        ],
        "stt-openai" => vec![
            STTModel { id: "model-oai-whisper3".to_string(), provider_id: provider_id.clone(), model_id: "whisper-1".to_string(), name: "Whisper v3".to_string(), languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "ja".to_string()], word_error_rate: Some(11.6), latency_ms: Some(500), supports_streaming: true, supports_diarization: false, quality_tier: "premium".to_string(), enabled: true },
        ],
        "stt-assemblyai" => vec![
            STTModel { id: "model-aai-best".to_string(), provider_id: provider_id.clone(), model_id: "best".to_string(), name: "Best".to_string(), languages: vec!["en".to_string()], word_error_rate: Some(14.5), latency_ms: Some(400), supports_streaming: true, supports_diarization: true, quality_tier: "premium".to_string(), enabled: true },
            STTModel { id: "model-aai-nano".to_string(), provider_id: provider_id.clone(), model_id: "nano".to_string(), name: "Nano (Fast)".to_string(), languages: vec!["en".to_string()], word_error_rate: Some(18.0), latency_ms: Some(150), supports_streaming: true, supports_diarization: false, quality_tier: "standard".to_string(), enabled: true },
        ],
        "stt-google" => vec![
            STTModel { id: "model-gc-chirp".to_string(), provider_id: provider_id.clone(), model_id: "chirp".to_string(), name: "Chirp".to_string(), languages: vec!["en".to_string(), "es".to_string(), "fr".to_string()], word_error_rate: Some(11.6), latency_ms: Some(350), supports_streaming: true, supports_diarization: true, quality_tier: "premium".to_string(), enabled: true },
        ],
        "stt-minimax" => vec![
            STTModel { id: "model-mm-standard".to_string(), provider_id: provider_id.clone(), model_id: "speech-01".to_string(), name: "Speech 01".to_string(), languages: vec!["zh".to_string(), "ar".to_string(), "en".to_string()], word_error_rate: Some(10.0), latency_ms: Some(200), supports_streaming: true, supports_diarization: false, quality_tier: "premium".to_string(), enabled: true },
        ],
        _ => vec![],
    };
    
    Json(models)
}

/// Transcribe audio to text
pub async fn transcribe_audio(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<STTRequest>,
) -> Json<STTResponse> {
    // Demo response - in production, call actual STT provider
    Json(STTResponse {
        text: "This is a sample transcription. In production, this would be the actual transcribed text.".to_string(),
        confidence: 0.95,
        words: vec![
            WordTiming { word: "This".to_string(), start: 0.0, end: 0.3, confidence: 0.98 },
            WordTiming { word: "is".to_string(), start: 0.3, end: 0.5, confidence: 0.99 },
            WordTiming { word: "a".to_string(), start: 0.5, end: 0.6, confidence: 0.99 },
            WordTiming { word: "sample".to_string(), start: 0.6, end: 1.1, confidence: 0.97 },
            WordTiming { word: "transcription".to_string(), start: 1.1, end: 1.9, confidence: 0.96 },
        ],
        duration_seconds: 5.0,
        language: request.language,
        provider: "deepgram".to_string(),
        model: request.model,
        speakers: if request.diarization {
            Some(vec![
                Speaker { id: 0, segments: vec![SpeakerSegment { start: 0.0, end: 2.5, text: "This is a sample".to_string() }] },
                Speaker { id: 1, segments: vec![SpeakerSegment { start: 2.5, end: 5.0, text: "transcription.".to_string() }] },
            ])
        } else {
            None
        },
    })
}

// ============================================================================
// WebRTC Handlers
// ============================================================================

/// Create a WebRTC session for real-time voice
pub async fn create_webrtc_session(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<CreateWebRTCSession>,
) -> Json<WebRTCSession> {
    let session_id = uuid::Uuid::new_v4().to_string();
    
    // Default ICE servers (STUN/TURN)
    let ice_servers = vec![
        IceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            username: None,
            credential: None,
        },
        IceServer {
            urls: vec!["stun:stun1.l.google.com:19302".to_string()],
            username: None,
            credential: None,
        },
    ];
    
    Json(WebRTCSession {
        id: session_id,
        session_type: request.session_type,
        status: "pending".to_string(),
        ice_servers,
        stt_provider: request.stt_provider_id,
        tts_provider: request.tts_provider_id,
    })
}

/// Get WebRTC session status
pub async fn get_webrtc_session(
    State(_state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Json<WebRTCSession> {
    Json(WebRTCSession {
        id: session_id,
        session_type: "voice_chat".to_string(),
        status: "active".to_string(),
        ice_servers: vec![],
        stt_provider: Some("stt-deepgram".to_string()),
        tts_provider: Some("tts-elevenlabs".to_string()),
    })
}

#[derive(Debug, Deserialize)]
pub struct VoiceLanguageQuery {
    pub provider: Option<String>,
}

/// Get supported languages
pub async fn get_supported_languages(
    Query(_query): Query<VoiceLanguageQuery>,
) -> Json<Vec<serde_json::Value>> {
    Json(vec![
        serde_json::json!({"code": "en", "name": "English", "variants": ["en-US", "en-GB", "en-AU"]}),
        serde_json::json!({"code": "es", "name": "Spanish", "variants": ["es-ES", "es-MX", "es-AR"]}),
        serde_json::json!({"code": "fr", "name": "French", "variants": ["fr-FR", "fr-CA"]}),
        serde_json::json!({"code": "de", "name": "German", "variants": ["de-DE", "de-AT"]}),
        serde_json::json!({"code": "zh", "name": "Chinese", "variants": ["zh-CN", "zh-TW"]}),
        serde_json::json!({"code": "ar", "name": "Arabic", "variants": ["ar-SA", "ar-EG", "ar-AE"]}),
        serde_json::json!({"code": "ja", "name": "Japanese", "variants": ["ja-JP"]}),
        serde_json::json!({"code": "ko", "name": "Korean", "variants": ["ko-KR"]}),
        serde_json::json!({"code": "hi", "name": "Hindi", "variants": ["hi-IN"]}),
        serde_json::json!({"code": "pt", "name": "Portuguese", "variants": ["pt-BR", "pt-PT"]}),
        serde_json::json!({"code": "ru", "name": "Russian", "variants": ["ru-RU"]}),
        serde_json::json!({"code": "it", "name": "Italian", "variants": ["it-IT"]}),
    ])
}
