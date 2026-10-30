use crate::{
    agenda,
    dados::Dados,
    metas::Goal,
    organizacao::{Note, Task},
};
use serde_json::{json, Value};
struct Store {
    data: Option<Dados>,
    root: std::path::PathBuf,
}
impl Store {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lunes-work-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Self {
            data: Some(Dados::open(root.clone()).unwrap()),
            root,
        }
    }
    fn d(&self) -> &Dados {
        self.data.as_ref().unwrap()
    }
    fn run(&self, tool: &str, args: Value) -> Value {
        self.d()
            .execute_agenda("trabalho", tool, &args, None)
            .unwrap()
    }
    fn reopen(&mut self) {
        self.data.take();
        self.data = Some(Dados::open(self.root.clone()).unwrap());
    }
}
impl Drop for Store {
    fn drop(&mut self) {
        self.data.take();
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}
fn task() -> Task {
    Task {
        id: None,
        title: "Entrega".into(),
        priority: "alta".into(),
        due: "2026-10-29".into(),
        done: false,
        token: "00000000-0000-4000-8000-000000000001".into(),
    }
}
fn goal() -> Goal {
    Goal {
        id: None,
        title: "Entregar projeto".into(),
        description: "Critério de entrega".into(),
        due: Some("2026-12-01".into()),
        token: "00000000-0000-4000-8000-000000000002".into(),
    }
}
#[test]
fn work_only_persistence_lock_validation_and_stale_edit() {
    let mut s = Store::new();
    let d = s.d();
    assert!(Dados::open(s.root.clone()).is_err());
    let mut t = task();
    assert!(d.save_task("pessoal", &t).is_err());
    assert!(d.organization("estudos").is_err());
    assert!(d.goals("pessoal", None).is_err());
    t.due = "2026-02-30".into();
    assert!(d.save_task("trabalho", &t).is_err());
    t = task();
    let id = d.save_task("trabalho", &t).unwrap();
    assert_eq!(id, d.save_task("trabalho", &t).unwrap());
    t.id = Some(id);
    t.done = true;
    d.save_task_checked("trabalho", &t, Some(0)).unwrap();
    assert!(d.save_task_checked("trabalho", &t, Some(0)).is_err());
    let n = Note {
        id: None,
        title: "Ideia".into(),
        body: "Anotação local".into(),
        token: "00000000-0000-4000-8000-000000000003".into(),
    };
    let note = d.save_note("trabalho", &n).unwrap();
    d.save_theme("escuro").unwrap();
    assert!(d.save_theme("outro").is_err());
    s.reopen();
    assert_eq!(s.d().read_theme(None).unwrap(), "escuro");
    assert_eq!(
        s.d().organization("trabalho").unwrap()["tasks"][0]["done"],
        true
    );
    s.d().delete_record("trabalho", "note", note).unwrap();
    s.reopen();
    assert_eq!(s.d().organization("trabalho").unwrap()["notes"], json!([]));
}
#[test]
fn monthly_recurrence_retries_and_manual_exclusion_survive_restart() {
    let mut s = Store::new();
    {
        let mut c = s.d().connection.lock().unwrap();
        let tx = c.transaction().unwrap();
        tx.execute("INSERT INTO recurrence_rules(area,title,priority,frequency,start,token) VALUES ('trabalho','Mensal','alta','monthly','2024-01-31','00000000-0000-4000-8000-000000000004')",[]).unwrap();
        agenda::generate(&tx, "trabalho", "2024-05-31").unwrap();
        agenda::generate(&tx, "trabalho", "2024-05-31").unwrap();
        tx.commit().unwrap();
    }
    let rows = s.d().organization("trabalho").unwrap();
    let dates: Vec<_> = rows["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["due"].as_str().unwrap())
        .collect();
    assert_eq!(
        dates,
        vec![
            "2024-01-31",
            "2024-02-29",
            "2024-03-31",
            "2024-04-30",
            "2024-05-31"
        ]
    );
    let id = rows["tasks"][1]["id"].as_i64().unwrap();
    s.d().delete_record("trabalho", "task", id).unwrap();
    s.reopen();
    {
        let mut c = s.d().connection.lock().unwrap();
        let tx = c.transaction().unwrap();
        agenda::generate(&tx, "trabalho", "2024-05-31").unwrap();
        tx.commit().unwrap();
    }
    assert_eq!(
        s.d().organization("trabalho").unwrap()["tasks"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}
#[test]
fn meeting_requires_explicit_conversion_and_preserves_tasks_after_delete() {
    let mut s = Store::new();
    let args = json!({"id":null,"token":"00000000-0000-4000-8000-000000000005","title":"Planejamento","date":"2026-10-29","agenda":"Pauta","notes":"Notas","decisions":"Decisões","actions":[{"token":"00000000-0000-4000-8000-000000000006","title":"Próximo passo","priority":"alta","due":"2026-10-29"}]});
    let saved = s.run("salvar_reuniao", args.clone());
    let id = saved["meeting"].as_i64().unwrap();
    assert_eq!(s.d().organization("trabalho").unwrap()["tasks"], json!([]));
    let conversion = json!({"id":id,"action_token":"00000000-0000-4000-8000-000000000006","title":"Conferida","priority":"media","due":"2026-10-29"});
    let t = s.run("converter_reuniao", conversion.clone());
    let retry = s.run("converter_reuniao", conversion);
    assert_eq!(t["task"], retry["task"]);
    assert_eq!(retry["alreadyConverted"], true);
    let mut edit = args;
    edit["id"] = json!(id);
    s.run("salvar_reuniao", edit.clone());
    assert!(s
        .d()
        .execute_agenda("trabalho", "salvar_reuniao", &edit, Some(0))
        .is_err());
    s.run("excluir_reuniao", json!({"id":id}));
    s.reopen();
    assert_eq!(
        s.d().organization("trabalho").unwrap()["tasks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        s.d().agenda("trabalho", None).unwrap()["meetings"],
        json!([])
    );
}
#[test]
fn goal_progress_and_delete_preserve_shared_tasks() {
    let s = Store::new();
    let g = goal();
    let id = s.d().save_goal("trabalho", &g, None).unwrap();
    assert_eq!(id, s.d().save_goal("trabalho", &g, None).unwrap());
    assert_eq!(
        s.d().goals("trabalho", Some(id)).unwrap()["goals"][0]["total"],
        0
    );
    let mut t = task();
    let tid = s.d().save_goal_task("trabalho", id, &t, 1).unwrap();
    assert_eq!(tid, s.d().save_goal_task("trabalho", id, &t, 1).unwrap());
    t.id = Some(tid);
    t.done = true;
    s.d().save_task_checked("trabalho", &t, Some(0)).unwrap();
    assert_eq!(
        s.d().goals("trabalho", Some(id)).unwrap()["goals"][0]["progress"],
        100
    );
    assert!(s.d().delete_goal("trabalho", id, 99).is_err());
    s.d().delete_goal("trabalho", id, 1).unwrap();
    assert_eq!(
        s.d().organization("trabalho").unwrap()["tasks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn backups_restore_atomically_and_reject_foreign_or_modified_schema() {
    let mut s = Store::new();
    s.d().save_task("trabalho", &task()).unwrap();
    let b = s.d().export().unwrap();
    let path = s.root.join("backups").join(&b);
    s.d().delete_record("trabalho", "task", 1).unwrap();
    s.d().restore(&b).unwrap();
    assert_eq!(
        s.d().organization("trabalho").unwrap()["tasks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(s.d().restore("../lunes.sqlite3").is_err());
    let c = rusqlite::Connection::open(&path).unwrap();
    c.pragma_update(None, "application_id", 0x4c554e45).unwrap();
    drop(c);
    assert!(s.d().restore(&b).is_err());
    let fresh = s.d().export().unwrap();
    let c = rusqlite::Connection::open(s.root.join("backups").join(&fresh)).unwrap();
    c.execute_batch("CREATE TRIGGER tamper AFTER INSERT ON notes BEGIN DELETE FROM tasks; END;")
        .unwrap();
    drop(c);
    assert!(s.d().restore(&fresh).is_err());
    s.reopen();
    assert_eq!(
        s.d().organization("trabalho").unwrap()["tasks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
