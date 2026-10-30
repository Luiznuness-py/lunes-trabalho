use crate::dados::Dados;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
type Result<T> = std::result::Result<T, String>;
const DISK: &str =
    "Não foi possível salvar ou consultar seus registros. Verifique disco e permissões.";
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: Option<i64>,
    pub title: String,
    pub priority: String,
    pub due: String,
    pub done: bool,
    pub token: String,
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub id: Option<i64>,
    pub title: String,
    pub body: String,
    pub token: String,
}
pub fn title(s: &str) -> Result<()> {
    if s.trim().is_empty() || s.len() > 200 || s.contains('\0') {
        Err("Título obrigatório, até 200 bytes.".into())
    } else {
        Ok(())
    }
}
pub fn date(s: &str) -> Result<()> {
    let parts: Vec<_> = s.split('-').collect();
    if parts.len() != 3
        || s.len() != 10
        || parts[0].len() != 4
        || parts[1].len() != 2
        || parts[2].len() != 2
        || !parts.iter().all(|s| s.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("Data inválida. Use AAAA-MM-DD.".into());
    }
    let y: u32 = parts[0].parse().map_err(|_| "Data inválida.")?;
    let m: usize = parts[1].parse().map_err(|_| "Data inválida.")?;
    let d: u32 = parts[2].parse().map_err(|_| "Data inválida.")?;
    let days = [
        0,
        31,
        if y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400)) {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1900..=9999).contains(&y) || !(1..=12).contains(&m) || d == 0 || d > days[m] {
        Err("Data inexistente.".into())
    } else {
        Ok(())
    }
}
pub fn task_valid(t: &Task) -> Result<()> {
    if t.id.is_some_and(|id| id <= 0) {
        return Err("Identificador de tarefa inválido.".into());
    }
    title(&t.title)?;
    date(&t.due)?;
    if !matches!(t.priority.as_str(), "alta" | "media" | "baixa") || !valid_token(&t.token) {
        return Err("Prioridade ou identificador inválido.".into());
    }
    Ok(())
}
pub fn note_valid(n: &Note) -> Result<()> {
    if n.id.is_some_and(|id| id <= 0) {
        return Err("Identificador de nota inválido.".into());
    }
    title(&n.title)?;
    if n.body.trim().is_empty()
        || n.body.len() > 65536
        || n.body.contains('\0')
        || !valid_token(&n.token)
    {
        return Err("Ideia obrigatória, até 64 KiB, com identificador válido.".into());
    }
    Ok(())
}
pub fn personal_area(s: &str) -> Result<()> {
    if s == "trabalho" {
        Ok(())
    } else {
        Err("Operação disponível apenas em Trabalho.".into())
    }
}
pub(crate) fn task_view(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    Ok(
        json!({"id":r.get::<_,i64>(0)?,"title":r.get::<_,String>(1)?,"priority":r.get::<_,String>(2)?,"due":r.get::<_,String>(3)?,"done":r.get::<_,bool>(4)?,"token":r.get::<_,String>(5)?,"completedAt":r.get::<_,Option<i64>>(7)?,"scheduledDue":r.get::<_,Option<String>>(9)?,"ruleId":r.get::<_,Option<i64>>(10)?,"meetingId":r.get::<_,Option<i64>>(11)?,"revision":r.get::<_,i64>(12)?}),
    )
}
impl Dados {
    pub fn organization(&self, area: &str) -> Result<Value> {
        personal_area(area)?;
        let c = self.connection.lock().map_err(|_| DISK)?;
        let tasks=c.prepare("SELECT id,title,priority,due,done,token,cancelled,completed_at,cancelled_at,scheduled_due,rule_id,meeting_id,revision FROM tasks WHERE area=?1 AND deleted=0 ORDER BY done,cancelled,CASE WHEN due<strftime('%Y-%m-%d','now','-3 hours') THEN 0 WHEN due=strftime('%Y-%m-%d','now','-3 hours') THEN 1 ELSE 2 END,CASE priority WHEN 'alta' THEN 0 WHEN 'media' THEN 1 ELSE 2 END,due,id DESC").map_err(|_|DISK)?.query_map([area],task_view).map_err(|_|DISK)?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|DISK)?;
        let notes = c
            .prepare(
                "SELECT id,title,body,token FROM notes WHERE area=?1 ORDER BY id DESC LIMIT 200",
            )
            .map_err(|_| DISK)?
            .query_map([area], |r| {
                Ok(Note {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    body: r.get(2)?,
                    token: r.get(3)?,
                })
            })
            .map_err(|_| DISK)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| DISK)?;
        Ok(json!({"tasks":tasks,"notes":notes}))
    }
    #[cfg(test)]
    pub fn save_task(&self, area: &str, t: &Task) -> Result<i64> {
        self.save_task_checked(area, t, None)
    }
    pub fn save_task_checked(&self, area: &str, t: &Task, expected: Option<i64>) -> Result<i64> {
        personal_area(area)?;
        task_valid(t)?;
        let mut c = self.connection.lock().map_err(|_| DISK)?;
        let tx = c.transaction().map_err(|_| DISK)?;
        if let (Some(id), Some(revision)) = (t.id, expected) {
            let actual: (i64, String) = tx
                .query_row(
                    "SELECT revision,token FROM tasks WHERE id=?1 AND area=?2 AND deleted=0",
                    params![id, area],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(|_| "Tarefa ausente nesta área.")?;
            if actual != (revision, t.token.clone()) {
                return Err("A tarefa mudou. Reabra o editor antes de salvar.".into());
            }
        }
        let id = write_task(&tx, area, t)?;
        tx.commit().map_err(|_| DISK)?;
        Ok(id)
    }
    pub fn save_note(&self, area: &str, n: &Note) -> Result<i64> {
        personal_area(area)?;
        note_valid(n)?;
        let mut c = self.connection.lock().map_err(|_| DISK)?;
        let tx = c.transaction().map_err(|_| DISK)?;
        let id = write_note(&tx, area, n)?;
        tx.commit().map_err(|_| DISK)?;
        Ok(id)
    }
    pub fn delete_record(&self, area: &str, kind: &str, id: i64) -> Result<()> {
        let c = self.connection.lock().map_err(|_| DISK)?;
        write_delete_record(&c, area, kind, id)
    }
}
pub(crate) fn write_delete_record(
    c: &rusqlite::Connection,
    area: &str,
    kind: &str,
    id: i64,
) -> Result<()> {
    personal_area(area)?;
    let table = match kind {
        "task" => "tasks",
        "note" => "notes",
        _ => return Err("Tipo de registro inválido.".into()),
    };
    if kind == "task" {
        let recurring: bool = c
            .query_row(
                "SELECT rule_id IS NOT NULL FROM tasks WHERE id=?1 AND area=?2 AND deleted=0",
                params![id, area],
                |r| r.get(0),
            )
            .map_err(|_| "Tarefa ausente nesta área.")?;
        if recurring {
            c.execute(
                "UPDATE tasks SET deleted=1,deleted_reason='manual',revision=revision+1 WHERE id=?1 AND area=?2",
                params![id, area],
            )
            .map_err(|_| DISK)?;
            return Ok(());
        }
    }
    if c.execute(
        &format!("DELETE FROM {table} WHERE id=?1 AND area=?2"),
        params![id, area],
    )
    .map_err(|_| DISK)?
        != 1
    {
        return Err("Registro ausente nesta área.".into());
    }
    Ok(())
}

pub(crate) fn write_task(c: &rusqlite::Connection, area: &str, t: &Task) -> Result<i64> {
    personal_area(area)?;
    task_valid(t)?;
    if let Some(id) = t.id {
        let (old,cancelled):(bool,bool)=c.query_row("SELECT done,cancelled FROM tasks WHERE id=?1 AND area=?2 AND token=?3 AND deleted=0",params![id,area,t.token],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|_|"Tarefa ausente nesta área ou identificador desatualizado.")?;
        if cancelled && t.done {
            return Err("Retome a tarefa cancelada antes de concluir.".into());
        }
        c.execute("UPDATE tasks SET title=?3,priority=?4,due=?5,done=?6,completed_at=CASE WHEN ?6=1 AND done=0 THEN unixepoch() WHEN ?6=0 THEN NULL ELSE completed_at END,scheduled_due=CASE WHEN rule_id IS NULL THEN ?5 ELSE scheduled_due END,revision=revision+1 WHERE id=?1 AND area=?2",params![id,area,t.title.trim(),t.priority,t.due,t.done]).map_err(|_|DISK)?;
        if old != t.done {
            crate::agenda::event(c, area, id, if t.done { "completed" } else { "reopened" })?;
        }
        return Ok(id);
    }
    let inserted=c.execute("INSERT OR IGNORE INTO tasks(area,title,priority,due,done,token,scheduled_due,completed_at) VALUES (?1,?2,?3,?4,?5,?6,?4,CASE WHEN ?5 THEN unixepoch() ELSE NULL END)",params![area,t.title.trim(),t.priority,t.due,t.done,t.token]).map_err(|_|DISK)?;
    let id = c
        .query_row(
            "SELECT id FROM tasks WHERE token=?1 AND area=?2",
            params![t.token, area],
            |r| r.get(0),
        )
        .optional()
        .map_err(|_| DISK)?
        .ok_or("Identificador já utilizado em outra área.")?;
    if inserted == 1 && t.done {
        crate::agenda::event(c, area, id, "completed")?;
    }
    Ok(id)
}
pub(crate) fn write_note(c: &rusqlite::Connection, area: &str, n: &Note) -> Result<i64> {
    personal_area(area)?;
    note_valid(n)?;
    if let Some(id) = n.id {
        if c.execute(
            "UPDATE notes SET title=?3,body=?4,revision=revision+1 WHERE id=?1 AND area=?2",
            params![id, area, n.title.trim(), n.body],
        )
        .map_err(|_| DISK)?
            != 1
        {
            return Err("Nota ausente nesta área.".into());
        }
        return Ok(id);
    }
    c.execute(
        "INSERT OR IGNORE INTO notes(area,title,body,token) VALUES (?1,?2,?3,?4)",
        params![area, n.title.trim(), n.body, n.token],
    )
    .map_err(|_| DISK)?;
    c.query_row(
        "SELECT id FROM notes WHERE token=?1 AND area=?2",
        params![n.token, area],
        |r| r.get(0),
    )
    .optional()
    .map_err(|_| DISK)?
    .ok_or("Identificador já utilizado em outra área.".into())
}
pub fn validate_records(c: &rusqlite::Connection) -> rusqlite::Result<bool> {
    let tasks = c
        .prepare("SELECT id,area,title,priority,due,done,token FROM tasks")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(1)?,
                Task {
                    id: r.get(0)?,
                    title: r.get(2)?,
                    priority: r.get(3)?,
                    due: r.get(4)?,
                    done: r.get(5)?,
                    token: r.get(6)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if tasks
        .iter()
        .any(|(a, t)| personal_area(a).is_err() || task_valid(t).is_err())
    {
        return Ok(false);
    }
    let notes = c
        .prepare("SELECT id,area,title,body,token FROM notes")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(1)?,
                Note {
                    id: r.get(0)?,
                    title: r.get(2)?,
                    body: r.get(3)?,
                    token: r.get(4)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if notes
        .iter()
        .any(|(a, n)| personal_area(a).is_err() || note_valid(n).is_err())
    {
        return Ok(false);
    }
    Ok(true)
}

pub fn valid_token(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}

pub(crate) fn id(args: &Value, field: &str) -> Result<i64> {
    args[field]
        .as_i64()
        .filter(|v| *v > 0)
        .ok_or_else(|| format!("Identificador {field} inválido."))
}
pub(crate) fn text(args: &Value, field: &str) -> Result<String> {
    args[field]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("Campo {field} inválido."))
}
pub(crate) fn fields(args: &Value, allowed: &[&str], required: &[&str]) -> Result<()> {
    let object = args.as_object().ok_or("Argumentos inválidos.")?;
    if object.keys().any(|k| !allowed.contains(&k.as_str()))
        || required.iter().any(|k| !object.contains_key(*k))
    {
        return Err("Argumentos extras ou ausentes.".into());
    }
    Ok(())
}
