#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod agenda;
mod dados;
mod metas;
mod organizacao;
use serde_json::Value;
use tauri::{Emitter, Manager};
struct Storage(Result<dados::Dados, String>);
impl Storage {
    fn get(&self) -> Result<&dados::Dados, String> {
        self.0.as_ref().map_err(Clone::clone)
    }
}
#[tauri::command]
fn ler_tema(state: tauri::State<Storage>, legado: Option<String>) -> Result<String, String> {
    state.get()?.read_theme(legado.as_deref())
}
#[tauri::command]
fn salvar_tema(
    app: tauri::AppHandle,
    state: tauri::State<Storage>,
    tema: String,
) -> Result<(), String> {
    state.get()?.save_theme(&tema)?;
    app.emit("tema-alterado", &tema)
        .map_err(|_| "Tema salvo, mas a sincronização das janelas falhou.".into())
}
#[tauri::command]
fn ler_organizacao(
    storage: tauri::State<Storage>,
    area: String,
    through: Option<String>,
) -> Result<Value, String> {
    storage.get()?.agenda(&area, through.as_deref())
}
#[tauri::command]
fn executar_agenda(
    storage: tauri::State<Storage>,
    area: String,
    tool: String,
    args: Value,
    expected_revision: Option<i64>,
) -> Result<Value, String> {
    storage
        .get()?
        .execute_agenda(&area, &tool, &args, expected_revision)
}
#[tauri::command]
fn ler_metas(
    storage: tauri::State<Storage>,
    area: String,
    goal_id: Option<i64>,
) -> Result<Value, String> {
    storage.get()?.goals(&area, goal_id)
}
#[tauri::command]
fn salvar_meta(
    app: tauri::AppHandle,
    storage: tauri::State<Storage>,
    area: String,
    goal: metas::Goal,
    expected_revision: Option<i64>,
) -> Result<i64, String> {
    let id = storage.get()?.save_goal(&area, &goal, expected_revision)?;
    let _ = app.emit("metas-alteradas", &area);
    Ok(id)
}
#[tauri::command]
fn excluir_meta(
    app: tauri::AppHandle,
    storage: tauri::State<Storage>,
    area: String,
    id: i64,
    expected_revision: i64,
) -> Result<(), String> {
    storage.get()?.delete_goal(&area, id, expected_revision)?;
    let _ = app.emit("metas-alteradas", &area);
    Ok(())
}
#[tauri::command]
fn salvar_tarefa_meta(
    app: tauri::AppHandle,
    storage: tauri::State<Storage>,
    area: String,
    goal_id: i64,
    task: organizacao::Task,
    expected_revision: i64,
) -> Result<i64, String> {
    let id = storage
        .get()?
        .save_goal_task(&area, goal_id, &task, expected_revision)?;
    let _ = app.emit("metas-alteradas", &area);
    Ok(id)
}
#[tauri::command]
fn salvar_tarefa(
    app: tauri::AppHandle,
    storage: tauri::State<Storage>,
    area: String,
    task: organizacao::Task,
    expected_revision: Option<i64>,
) -> Result<i64, String> {
    let id = storage
        .get()?
        .save_task_checked(&area, &task, expected_revision)?;
    let _ = app.emit("metas-alteradas", &area);
    Ok(id)
}
#[tauri::command]
fn salvar_nota(
    storage: tauri::State<Storage>,
    area: String,
    note: organizacao::Note,
) -> Result<i64, String> {
    storage.get()?.save_note(&area, &note)
}
#[tauri::command]
fn excluir_registro(
    app: tauri::AppHandle,
    storage: tauri::State<Storage>,
    area: String,
    kind: String,
    id: i64,
) -> Result<(), String> {
    storage.get()?.delete_record(&area, &kind, id)?;
    let _ = app.emit("metas-alteradas", &area);
    Ok(())
}
#[tauri::command]
fn exportar_dados(storage: tauri::State<Storage>) -> Result<String, String> {
    storage.get()?.export()
}
#[tauri::command]
fn restaurar_dados(storage: tauri::State<Storage>, backup: String) -> Result<(), String> {
    storage.get()?.restore(&backup)
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let storage = app
                .path()
                .app_local_data_dir()
                .map_err(|_| "Diretório de dados indisponível.".to_string())
                .and_then(dados::Dados::open);
            app.manage(Storage(storage));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ler_tema,
            salvar_tema,
            ler_organizacao,
            executar_agenda,
            ler_metas,
            salvar_meta,
            excluir_meta,
            salvar_tarefa_meta,
            salvar_tarefa,
            salvar_nota,
            excluir_registro,
            exportar_dados,
            restaurar_dados
        ])
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar o Lunes Trabalho.");
}

#[cfg(test)]
mod work_tests;
