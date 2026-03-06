//! Orquestração de subprocessos para FFmpeg e FFprobe.
//!
//! Este módulo garante que comandos de CLI sejam executados de forma segura,
//! com limites de tempo (timeouts) e limpeza de processos órfãos.
use crate::core::error::{AppError, AppResult};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tauri::Manager;
use wait_timeout::ChildExt;

/// Caminhos resolvidos para os binários de vídeo.
pub struct TranscodingTools {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

/// Resolve os caminhos para as ferramentas de transcoding.
///
/// Tenta localizar os binários na pasta de recursos do Tauri, no diretório de debug
/// ou no PATH global do sistema.
pub fn resolve_transcoding_tools<R: tauri::Runtime>(
    app_handle: Option<&tauri::AppHandle<R>>,
) -> AppResult<TranscodingTools> {
    let ffmpeg = find_binary(app_handle, "ffmpeg")
        .ok_or_else(|| AppError::Transcoding("FFmpeg binary not found".to_string()))?;
    let ffprobe = find_binary(app_handle, "ffprobe")
        .ok_or_else(|| AppError::Transcoding("FFprobe binary not found".to_string()))?;
    Ok(TranscodingTools { ffmpeg, ffprobe })
}

/// Tenta encontrar um binário específico no sistema.
fn find_binary<R: tauri::Runtime>(
    app_handle: Option<&tauri::AppHandle<R>>,
    name: &str,
) -> Option<PathBuf> {
    let binary_name = if cfg!(target_os = "windows") {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };
    // 1. Tenta via Tauri Resource Dir (Bundled)
    if let Some(handle) = app_handle {
        if let Ok(resource_dir) = handle.path().resource_dir() {
            let bundled_path = resource_dir.join("ffmpeg").join(&binary_name);
            if bundled_path.exists() {
                return Some(bundled_path);
            }
        }
    }
    // 2. Tenta via caminhos relativos de desenvolvimento (src-tauri/ffmpeg)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(target_dir) = exe_path.parent() {
            if let Some(debug_dir) = target_dir.parent() {
                if let Some(src_tauri) = debug_dir.parent() {
                    let bundled_path = src_tauri.join("ffmpeg").join(&binary_name);
                    if bundled_path.exists() {
                        return Some(bundled_path);
                    }
                }
            }
        }
    }
    // 3. Tenta via PATH global do sistema
    if Command::new(name)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Some(PathBuf::from(name));
    }
    None
}

/// Executa um comando com um timeout rigoroso.
///
/// # Arguments
/// * `cmd` - O comando pronto para execução.
/// * `timeout_secs` - Tempo máximo de execução em segundos.
///
/// # Errors
/// Retorna `AppError::Transcoding` se o comando falhar, expirar ou emitir erro.
pub fn run_command_with_timeout(
    mut command: Command,
    timeout_secs: u64,
) -> AppResult<std::process::Output> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Transcoding(format!("Failed to spawn process: {}", e)))?;
    match child.wait_timeout(Duration::from_secs(timeout_secs))? {
        Some(status) => {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut s) = child.stdout {
                s.read_to_end(&mut stdout).ok();
            }
            if let Some(mut s) = child.stderr {
                s.read_to_end(&mut stderr).ok();
            }
            Ok(std::process::Output {
                status,
                stdout,
                stderr,
            })
        }
        None => {
            // Processo expirou. Matamos para evitar zumbis.
            child.kill().ok();
            Err(AppError::Transcoding(format!(
                "Process execution timed out after {}s",
                timeout_secs
            )))
        }
    }
}

/// Verifica a disponibilidade dos binários de transcoding.
pub fn check_transcoding_availability() -> bool {
    // Verificação rápida apenas de existência estática
    find_binary::<tauri::Wry>(None, "ffmpeg").is_some()
        && find_binary::<tauri::Wry>(None, "ffprobe").is_some()
}

/// Unit tests for the transcoding module.
#[cfg(test)]
mod tests {
    use super::*;

    /// Testa a detecção de binários de transcoding.
    #[test]
    fn test_binary_detection() {
        // No CI/Local deve encontrar se instalado
        let tools = resolve_transcoding_tools::<tauri::Wry>(None);
        assert!(
            tools.is_ok(),
            "FFmpeg/FFprobe should be available for tests"
        );
    }

    /// Testa a execução de um comando com timeout.
    #[test]
    fn test_timeout_execution() {
        let mut cmd = Command::new("sleep");
        cmd.arg("2");
        // Deve falhar com timeout de 1s
        let result = run_command_with_timeout(cmd, 1);
        assert!(result.is_err());
    }
}
