use crate::{
    dados::Dados,
    organizacao::{self, Task},
    organizacao::{fields, id, text},
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
type Result<T> = std::result::Result<T, String>;
const DISK: &str = "Não foi possível acessar a agenda. Verifique disco e permissões.";
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Prepared {
    pub tool: String,
    pub args: Value,
    pub revision: i64,
    pub token: String,
    pub label: String,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MeetingAction {
    pub token: String,
    pub title: String,
    pub priority: String,
    pub due: String,
}
#[derive(Clone, Copy)]
struct Day {
    y: u32,
    m: usize,
    d: u32,
}
impl Day {
    fn parse(s: &str) -> Result<Self> {
        organizacao::date(s)?;
        Ok(Self {
            y: s[..4].parse().unwrap(),
            m: s[5..7].parse().unwrap(),
            d: s[8..].parse().unwrap(),
        })
    }
    fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
    fn days(self) -> u32 {
        [
            0,
            31,
            if self.y.is_multiple_of(4)
                && (!self.y.is_multiple_of(100) || self.y.is_multiple_of(400))
            {
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
        ][self.m]
    }
    fn next(mut self) -> Result<Self> {
        if self.d < self.days() {
            self.d += 1;
        } else {
            self.d = 1;
            if self.m == 12 {
                self.y += 1;
                self.m = 1
            } else {
                self.m += 1;
            }
        }
        if self.y > 9999 {
            return Err("Recorrência ultrapassa o calendário disponível.".into());
        }
        Ok(self)
    }
    fn add(mut self, n: usize) -> Result<Self> {
        for _ in 0..n {
            self = self.next()?;
        }
        Ok(self)
    }
}
pub fn today(c: &Connection) -> Result<String> {
    c.query_row("SELECT strftime('%Y-%m-%d','now','-3 hours')", [], |r| {
        r.get(0)
    })
    .map_err(|_| DISK.into())
}
fn uuid(c: &Connection) -> Result<String> {
    let v: String = c
        .query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))
        .map_err(|_| DISK)?;
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &v[..8],
        &v[8..12],
        &v[12..16],
        &v[16..20],
        &v[20..]
    ))
}
fn body(v: &Value, key: &str, max: usize, required: bool) -> Result<()> {
    let s = v[key].as_str().ok_or("Texto inválido.")?;
    if s.len() > max || s.contains('\0') || (required && s.trim().is_empty()) {
        return Err(format!(
            "{key}: texto inválido ou limite de {max} bytes excedido."
        ));
    }
    Ok(())
}
fn task_fields(a: &Value) -> Result<()> {
    organizacao::title(&text(a, "title")?)?;
    organizacao::date(&text(a, "due")?)?;
    if !matches!(a["priority"].as_str(), Some("alta" | "media" | "baixa")) {
        return Err("Prioridade inválida.".into());
    }
    Ok(())
}
fn actions(a: &Value) -> Result<Vec<MeetingAction>> {
    let rows: Vec<MeetingAction> = serde_json::from_value(a["actions"].clone())
        .map_err(|_| "Propostas de reunião inválidas.")?;
    if rows.len() > 30 {
        return Err("Use no máximo 30 propostas por reunião.".into());
    }
    let mut tokens = std::collections::HashSet::new();
    for r in &rows {
        if !organizacao::valid_token(&r.token) || !tokens.insert(&r.token) {
            return Err("Identificador de proposta inválido ou repetido.".into());
        }
        task_fields(&json!({"title":r.title,"priority":r.priority,"due":r.due}))?;
    }
    Ok(rows)
}
pub fn validate_args(tool: &str, a: &Value) -> Result<()> {
    if let Some(token) = a.get("token") {
        if !token.as_str().is_some_and(organizacao::valid_token) {
            return Err("Identificador de criação inválido.".into());
        }
    }
    match tool {
        "criar_recorrencia" => {
            fields(
                a,
                &["title", "priority", "due", "frequency", "until", "token"],
                &["title", "priority", "due", "frequency"],
            )?;
            task_fields(a)?;
            if !matches!(
                a["frequency"].as_str(),
                Some("daily" | "weekly" | "monthly")
            ) {
                return Err("Repetição inválida.".into());
            }
            if let Some(s) = a.get("until").filter(|v| !v.is_null()) {
                let s = s.as_str().ok_or("Data final inválida.")?;
                organizacao::date(s)?;
                if s < a["due"].as_str().unwrap() {
                    return Err("Data final anterior ao início.".into());
                }
            }
        }
        "editar_recorrencia" => {
            fields(
                a,
                &["id", "title", "priority", "until"],
                &["id", "title", "priority", "until"],
            )?;
            id(a, "id")?;
            organizacao::title(&text(a, "title")?)?;
            if !matches!(a["priority"].as_str(), Some("alta" | "media" | "baixa")) {
                return Err("Prioridade inválida.".into());
            }
            if !a["until"].is_null() {
                organizacao::date(&text(a, "until")?)?;
            }
        }
        "alterar_recorrencia" => {
            fields(a, &["id", "active"], &["id", "active"])?;
            id(a, "id")?;
            a["active"]
                .as_bool()
                .ok_or("Situação da recorrência inválida.")?;
        }
        "cancelar_tarefa" => {
            fields(a, &["id", "cancelled"], &["id", "cancelled"])?;
            id(a, "id")?;
            a["cancelled"].as_bool().ok_or("Cancelamento inválido.")?;
        }
        "salvar_journal" => {
            fields(a, &["date", "body"], &["date", "body"])?;
            organizacao::date(&text(a, "date")?)?;
            body(a, "body", 65536, true)?;
        }
        "excluir_journal" => {
            fields(a, &["date"], &["date"])?;
            organizacao::date(&text(a, "date")?)?;
        }
        "salvar_reuniao" => {
            fields(
                a,
                &[
                    "id",
                    "title",
                    "date",
                    "agenda",
                    "notes",
                    "decisions",
                    "actions",
                    "token",
                ],
                &[
                    "id",
                    "title",
                    "date",
                    "agenda",
                    "notes",
                    "decisions",
                    "actions",
                ],
            )?;
            if !a["id"].is_null() {
                id(a, "id")?;
            }
            organizacao::title(&text(a, "title")?)?;
            organizacao::date(&text(a, "date")?)?;
            for key in ["agenda", "notes", "decisions"] {
                body(a, key, 16000, false)?;
            }
            actions(a)?;
        }
        "excluir_reuniao" => {
            fields(a, &["id"], &["id"])?;
            id(a, "id")?;
        }
        "converter_reuniao" => {
            fields(
                a,
                &["id", "action_token", "title", "priority", "due"],
                &["id", "action_token", "title", "priority", "due"],
            )?;
            id(a, "id")?;
            task_fields(a)?;
            if !organizacao::valid_token(&text(a, "action_token")?) {
                return Err("Proposta inválida.".into());
            }
        }
        _ => return Err("Ferramenta de agenda não permitida.".into()),
    }
    Ok(())
}
fn table(tool: &str) -> Option<&'static str> {
    match tool {
        "editar_recorrencia" | "alterar_recorrencia" => Some("recurrence_rules"),
        "cancelar_tarefa" => Some("tasks"),
        "salvar_reuniao" | "excluir_reuniao" | "converter_reuniao" => Some("meetings"),
        _ => None,
    }
}
pub fn prepare(c: &Connection, area: &str, tool: &str, a: &Value) -> Result<Prepared> {
    organizacao::personal_area(area)?;
    if tool == "cancelar_tarefa" {
        return Err(
            "Cancelamento de tarefas não existe. Use excluir_tarefa para remover o registro."
                .into(),
        );
    }
    validate_args(tool, a)?;
    if tool.contains("reuniao") && area != "trabalho" {
        return Err("Reuniões ficam somente em Trabalho.".into());
    }
    let mut revision = -1;
    let mut token = a["token"]
        .as_str()
        .map(str::to_owned)
        .map(Ok)
        .unwrap_or_else(|| uuid(c))?;
    let mut label = a["title"]
        .as_str()
        .or(a["date"].as_str())
        .unwrap_or("Agenda")
        .to_owned();
    if let Some(table) = table(tool) {
        if !a["id"].is_null() {
            let i = id(a, "id")?;
            let row: (i64, String, String) = c
                .query_row(
                    &format!(
                        "SELECT revision,token,title FROM {table} WHERE id=?1 AND area=?2{}",
                        if table == "tasks" {
                            " AND deleted=0"
                        } else {
                            ""
                        }
                    ),
                    params![i, area],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .map_err(|_| "Registro ausente nesta área.")?;
            revision = row.0;
            token = row.1;
            if a.get("title").is_none() {
                label = row.2;
            }
        }
    }
    if tool.ends_with("journal") {
        revision = c
            .query_row(
                "SELECT revision FROM journal WHERE area=?1 AND date=?2",
                params![area, text(a, "date")?],
                |r| r.get(0),
            )
            .optional()
            .map_err(|_| DISK)?
            .unwrap_or(-1);
        if tool == "excluir_journal" && revision == -1 {
            return Err("Journal ausente nesta data.".into());
        }
    }
    Ok(Prepared {
        tool: tool.into(),
        args: a.clone(),
        revision,
        token,
        label,
    })
}
impl Prepared {
    pub fn valid(&self) -> bool {
        self.revision >= -1
            && organizacao::valid_token(&self.token)
            && organizacao::title(&self.label).is_ok()
            && validate_args(&self.tool, &self.args).is_ok()
    }
    pub fn apply(&self, c: &Connection, area: &str) -> Result<Value> {
        if self.tool == "cancelar_tarefa" {
            return Err("Cancelamento de tarefas não existe. Exclua ou recrie a tarefa.".into());
        }
        if !self.valid() {
            return Err("Operação de agenda inválida.".into());
        }
        organizacao::personal_area(area)?;
        if self.tool.contains("reuniao") && area != "trabalho" {
            return Err("Reunião pertence a Trabalho.".into());
        }
        let a = &self.args;
        if let Some(table) = table(&self.tool) {
            if !a["id"].is_null() {
                let current: Option<(i64, String)> = c
                    .query_row(
                        &format!(
                            "SELECT revision,token FROM {table} WHERE id=?1 AND area=?2{}",
                            if table == "tasks" {
                                " AND deleted=0"
                            } else {
                                ""
                            }
                        ),
                        params![id(a, "id")?, area],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .optional()
                    .map_err(|_| DISK)?;
                if current != Some((self.revision, self.token.clone())) {
                    return Err(
                        "O registro mudou. Reabra o editor ou prepare outra proposta.".into(),
                    );
                }
            }
        }
        if self.tool.ends_with("journal") {
            let rev = c
                .query_row(
                    "SELECT revision FROM journal WHERE area=?1 AND date=?2",
                    params![area, text(a, "date")?],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
                .map_err(|_| DISK)?
                .unwrap_or(-1);
            if rev != self.revision {
                return Err("O Journal mudou. Atualize antes de salvar.".into());
            }
        }
        match self.tool.as_str() {
            "criar_recorrencia" => {
                if let Some(i) = c
                    .query_row(
                        "SELECT id FROM recurrence_rules WHERE area=?1 AND token=?2",
                        params![area, self.token],
                        |r| r.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(|_| DISK)?
                {
                    let matches:bool=c.query_row("SELECT title=?3 AND priority=?4 AND frequency=?5 AND start=?6 AND until_date IS ?7 FROM recurrence_rules WHERE id=?1 AND area=?2",params![i,area,text(a,"title")?.trim(),text(a,"priority")?,text(a,"frequency")?,text(a,"due")?,a["until"].as_str()],|r|r.get(0)).map_err(|_|DISK)?;
                    if !matches {
                        return Err(
                            "Esta criação já foi salva com outros dados. Atualize a lista.".into(),
                        );
                    }
                    return Ok(json!({"rule":i}));
                }
                let count: i64 = c
                    .query_row(
                        "SELECT count(*) FROM recurrence_rules WHERE area=?1 AND active=1",
                        [area],
                        |r| r.get(0),
                    )
                    .map_err(|_| DISK)?;
                if count >= 100 {
                    return Err("Limite de 100 recorrências ativas nesta área.".into());
                }
                let start = text(a, "due")?;
                let now = Day::parse(&today(c)?)?;
                let oldest = format!(
                    "{:04}-{:02}-{:02}",
                    now.y.saturating_sub(10),
                    now.m,
                    now.d.min(28)
                );
                if start < oldest || start > now.add(366)?.iso() {
                    return Err(
                        "Início deve estar entre os últimos 10 anos e os próximos 12 meses.".into(),
                    );
                }
                c.execute("INSERT INTO recurrence_rules(area,title,priority,frequency,start,until_date,token) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![area,text(a,"title")?.trim(),text(a,"priority")?,text(a,"frequency")?,start,a["until"].as_str(),self.token]).map_err(|_|DISK)?;
                let rule = c.last_insert_rowid();
                generate(c, area, &now.add(90)?.iso())?;
                Ok(json!({"rule":rule}))
            }
            "editar_recorrencia" => {
                let i = id(a, "id")?;
                let start: String = c
                    .query_row("SELECT start FROM recurrence_rules WHERE id=?1", [i], |r| {
                        r.get(0)
                    })
                    .map_err(|_| DISK)?;
                if a["until"].as_str().is_some_and(|v| v < start.as_str()) {
                    return Err("Data final anterior ao início.".into());
                }
                c.execute("UPDATE recurrence_rules SET title=?2,priority=?3,until_date=?4,revision=revision+1 WHERE id=?1",params![i,text(a,"title")?.trim(),text(a,"priority")?,a["until"].as_str()]).map_err(|_|DISK)?;
                c.execute("UPDATE tasks SET title=?2,priority=?3,revision=revision+1 WHERE rule_id=?1 AND done=0 AND (deleted=0 OR deleted_reason='rule') AND scheduled_due>strftime('%Y-%m-%d','now','-3 hours') AND due>strftime('%Y-%m-%d','now','-3 hours')",params![i,text(a,"title")?.trim(),text(a,"priority")?]).map_err(|_|DISK)?;
                if let Some(end) = a["until"].as_str() {
                    remove_future(c, area, i, end)?;
                }
                prepare_future(c, area, i)?;
                Ok(json!({"rule":i}))
            }
            "alterar_recorrencia" => {
                let i = id(a, "id")?;
                let active = a["active"].as_bool().unwrap();
                c.execute(
                    "UPDATE recurrence_rules SET active=?2,revision=revision+1 WHERE id=?1",
                    params![i, active],
                )
                .map_err(|_| DISK)?;
                if !active {
                    remove_future(c, area, i, &today(c)?)?;
                } else {
                    prepare_future(c, area, i)?;
                }
                Ok(json!({"rule":i}))
            }
            "salvar_journal" => {
                c.execute("INSERT INTO journal(area,date,body) VALUES (?1,?2,?3) ON CONFLICT(area,date) DO UPDATE SET body=excluded.body,revision=journal.revision+1,updated_at=unixepoch()",params![area,text(a,"date")?,text(a,"body")?]).map_err(|_|DISK)?;
                Ok(json!({"journal":text(a,"date")?}))
            }
            "excluir_journal" => {
                c.execute(
                    "DELETE FROM journal WHERE area=?1 AND date=?2",
                    params![area, text(a, "date")?],
                )
                .map_err(|_| DISK)?;
                Ok(json!({"deleted_journal":text(a,"date")?}))
            }
            "salvar_reuniao" => {
                let rows = actions(a)?;
                let packed = serde_json::to_string(&rows).map_err(|_| DISK)?;
                let i = if a["id"].is_null() {
                    if let Some(i) = c
                        .query_row(
                            "SELECT id FROM meetings WHERE area=?1 AND token=?2",
                            params![area, self.token],
                            |r| r.get::<_, i64>(0),
                        )
                        .optional()
                        .map_err(|_| DISK)?
                    {
                        let matches:bool=c.query_row("SELECT title=?3 AND date=?4 AND agenda=?5 AND notes=?6 AND decisions=?7 AND actions=?8 FROM meetings WHERE id=?1 AND area=?2",params![i,area,text(a,"title")?.trim(),text(a,"date")?,text(a,"agenda")?,text(a,"notes")?,text(a,"decisions")?,packed],|r|r.get(0)).map_err(|_|DISK)?;
                        if !matches {
                            return Err(
                                "Esta reunião já foi salva com outros dados. Atualize a lista."
                                    .into(),
                            );
                        }
                        return Ok(json!({"meeting":i}));
                    }
                    c.execute("INSERT INTO meetings(area,title,date,agenda,notes,decisions,actions,token) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",params![area,text(a,"title")?.trim(),text(a,"date")?,text(a,"agenda")?,text(a,"notes")?,text(a,"decisions")?,packed,self.token]).map_err(|_|DISK)?;
                    c.last_insert_rowid()
                } else {
                    let i = id(a, "id")?;
                    let old: String = c
                        .query_row("SELECT actions FROM meetings WHERE id=?1", [i], |r| {
                            r.get(0)
                        })
                        .map_err(|_| DISK)?;
                    let old: Vec<MeetingAction> = serde_json::from_str(&old).map_err(|_| DISK)?;
                    for action in old {
                        let converted:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM meeting_conversions WHERE meeting=?1 AND action_token=?2)",params![i,action.token],|r|r.get(0)).map_err(|_|DISK)?;
                        if converted && !rows.contains(&action) {
                            return Err("Propostas já convertidas não podem ser alteradas ou removidas; edite a tarefa criada.".into());
                        }
                    }
                    c.execute("UPDATE meetings SET title=?2,date=?3,agenda=?4,notes=?5,decisions=?6,actions=?7,revision=revision+1 WHERE id=?1",params![i,text(a,"title")?.trim(),text(a,"date")?,text(a,"agenda")?,text(a,"notes")?,text(a,"decisions")?,packed]).map_err(|_|DISK)?;
                    i
                };
                Ok(json!({"meeting":i}))
            }
            "excluir_reuniao" => {
                c.execute(
                    "DELETE FROM meetings WHERE id=?1 AND area=?2",
                    params![id(a, "id")?, area],
                )
                .map_err(|_| DISK)?;
                Ok(json!({"deleted_meeting":id(a,"id")?}))
            }
            "converter_reuniao" => {
                let i = id(a, "id")?;
                let token = text(a, "action_token")?;
                let old: String = c
                    .query_row("SELECT actions FROM meetings WHERE id=?1", [i], |r| {
                        r.get(0)
                    })
                    .map_err(|_| DISK)?;
                let rows: Vec<MeetingAction> = serde_json::from_str(&old).map_err(|_| DISK)?;
                if !rows.iter().any(|a| a.token == token) {
                    return Err("Proposta ausente nesta reunião.".into());
                }
                let existing: Option<Option<i64>> = c
                    .query_row(
                        "SELECT task FROM meeting_conversions WHERE meeting=?1 AND action_token=?2",
                        params![i, token],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(|_| DISK)?;
                if let Some(task) = existing {
                    return Ok(json!({"task":task,"alreadyConverted":true}));
                }
                let task = Task {
                    id: None,
                    title: text(a, "title")?,
                    priority: text(a, "priority")?,
                    due: text(a, "due")?,
                    done: false,
                    token: uuid(c)?,
                };
                let task = organizacao::write_task(c, area, &task)?;
                c.execute(
                    "UPDATE tasks SET meeting_id=?2 WHERE id=?1",
                    params![task, i],
                )
                .map_err(|_| DISK)?;
                c.execute(
                    "INSERT INTO meeting_conversions(meeting,action_token,task) VALUES (?1,?2,?3)",
                    params![i, token, task],
                )
                .map_err(|_| DISK)?;
                Ok(json!({"task":task}))
            }
            _ => Err("Operação inválida.".into()),
        }
    }
}
fn remove_future(c: &Connection, area: &str, rule: i64, after: &str) -> Result<()> {
    c.execute("UPDATE tasks SET deleted=1,deleted_reason='rule',revision=revision+1 WHERE area=?1 AND rule_id=?2 AND scheduled_due>?3 AND due>strftime('%Y-%m-%d','now','-3 hours') AND done=0 AND deleted=0",params![area,rule,after]).map_err(|_|DISK)?;
    Ok(())
}
fn prepare_future(c: &Connection, area: &str, rule: i64) -> Result<()> {
    c.execute("UPDATE tasks SET deleted=0,deleted_reason=NULL,revision=revision+1 WHERE area=?1 AND rule_id=?2 AND deleted=1 AND deleted_reason='rule' AND due>=strftime('%Y-%m-%d','now','-3 hours') AND EXISTS(SELECT 1 FROM recurrence_rules r WHERE r.id=?2 AND r.active=1 AND (r.until_date IS NULL OR tasks.scheduled_due<=r.until_date))",params![area,rule]).map_err(|_|DISK)?;
    c.execute("UPDATE recurrence_rules SET generated_through=CASE WHEN generated_through<date('now','-3 hours','-1 day') THEN date('now','-3 hours','-1 day') ELSE generated_through END WHERE id=?1 AND area=?2 AND active=1",params![rule,area]).map_err(|_|DISK)?;
    Ok(())
}
pub fn event(c: &Connection, area: &str, id: i64, event: &str) -> Result<()> {
    c.execute("INSERT INTO task_events(task,area,title,due,event) SELECT id,area,title,due,?3 FROM tasks WHERE id=?1 AND area=?2",params![id,area,event]).map_err(|_|DISK)?;
    Ok(())
}
pub fn generate(c: &Connection, area: &str, through: &str) -> Result<()> {
    organizacao::personal_area(area)?;
    let now = Day::parse(&today(c)?)?;
    let horizon = Day::parse(through)?.iso();
    if horizon > now.add(366)?.iso() {
        return Err("Escolha um período até 12 meses à frente para gerar ocorrências.".into());
    }
    let rows=c.prepare("SELECT id,title,priority,frequency,start,until_date,generated_through FROM recurrence_rules WHERE area=?1 AND active=1").map_err(|_|DISK)?.query_map([area],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,Option<String>>(5)?,r.get::<_,Option<String>>(6)?))).map_err(|_|DISK)?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|DISK)?;
    let mut inserted = 0;
    for (rule, title, priority, freq, start, end, last) in rows {
        let anchor = Day::parse(&start)?;
        let stop = end.as_deref().map_or(horizon.as_str(), |v| v.min(&horizon));
        if last.as_deref().is_some_and(|v| v >= stop) || start.as_str() > stop {
            continue;
        }
        let mut month = 0;
        let mut day = if let Some(last) = last.as_deref() {
            let previous = Day::parse(last)?;
            if freq == "monthly" {
                month = ((previous.y - anchor.y) as usize) * 12 + previous.m - anchor.m;
                let m = anchor.m - 1 + month;
                let next = Day {
                    y: anchor.y + (m / 12) as u32,
                    m: m % 12 + 1,
                    d: 1,
                };
                let candidate = Day {
                    d: anchor.d.min(next.days()),
                    ..next
                };
                if candidate.iso().as_str() > last {
                    candidate
                } else {
                    month += 1;
                    let m = anchor.m - 1 + month;
                    let next = Day {
                        y: anchor.y + (m / 12) as u32,
                        m: m % 12 + 1,
                        d: 1,
                    };
                    Day {
                        d: anchor.d.min(next.days()),
                        ..next
                    }
                }
            } else if freq == "weekly" {
                let delta: i64 = c
                    .query_row(
                        "SELECT CAST(julianday(?1)-julianday(?2) AS INTEGER)",
                        params![last, start],
                        |r| r.get(0),
                    )
                    .map_err(|_| DISK)?;
                previous.add((7 - delta.rem_euclid(7)) as usize)?
            } else {
                previous.add(1)?
            }
        } else {
            anchor
        };
        while day.iso().as_str() <= stop {
            let due = day.iso();
            if last.as_deref().is_none_or(|v| due.as_str() > v) {
                inserted += 1;
                if inserted > 20000 {
                    return Err("Geração excedeu 20.000 ocorrências. Reduza o período ou pause recorrências antigas.".into());
                }
                let task = Task {
                    id: None,
                    title: title.clone(),
                    priority: priority.clone(),
                    due: due.clone(),
                    done: false,
                    token: uuid(c)?,
                };
                let existing: bool = c
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM tasks WHERE rule_id=?1 AND scheduled_due=?2)",
                        params![rule, due],
                        |r| r.get(0),
                    )
                    .map_err(|_| DISK)?;
                if !existing {
                    let id = organizacao::write_task(c, area, &task)?;
                    c.execute("UPDATE tasks SET rule_id=?2 WHERE id=?1", params![id, rule])
                        .map_err(|_| DISK)?;
                }
            }
            if due == "9999-12-31" {
                break;
            }
            day = if freq == "monthly" {
                month += 1;
                let m = anchor.m - 1 + month;
                let y = anchor.y + (m / 12) as u32;
                if y > 9999 {
                    break;
                }
                let next = Day {
                    y,
                    m: m % 12 + 1,
                    d: 1,
                };
                Day {
                    d: anchor.d.min(next.days()),
                    ..next
                }
            } else {
                day.add(if freq == "weekly" { 7 } else { 1 })?
            };
        }
        c.execute(
            "UPDATE recurrence_rules SET generated_through=?2 WHERE id=?1",
            params![rule, stop],
        )
        .map_err(|_| DISK)?;
    }
    let stored: i64 = c
        .query_row(
            "SELECT count(*) FROM tasks WHERE area=?1 AND deleted=0",
            [area],
            |r| r.get(0),
        )
        .map_err(|_| DISK)?;
    if stored > 50000 {
        return Err("A área atingiu 50.000 ocorrências. Reduza as repetições ou faça backup e remova ocorrências antigas antes de ampliar o período.".into());
    }
    Ok(())
}
impl Dados {
    pub fn execute_agenda(
        &self,
        area: &str,
        tool: &str,
        args: &Value,
        expected: Option<i64>,
    ) -> Result<Value> {
        let mut c = self.connection.lock().map_err(|_| DISK)?;
        let tx = c.transaction().map_err(|_| DISK)?;
        let operation = prepare(&tx, area, tool, args)?;
        if expected.is_some_and(|v| v != operation.revision) {
            return Err("O registro mudou. Reabra o editor antes de salvar.".into());
        }
        let result = operation.apply(&tx, area)?;
        tx.commit().map_err(|_| DISK)?;
        Ok(result)
    }
    pub fn agenda(&self, area: &str, through: Option<&str>) -> Result<Value> {
        let generation_error = {
            let mut c = self.connection.lock().map_err(|_| DISK)?;
            let tx = c.transaction().map_err(|_| DISK)?;
            let default = Day::parse(&today(&tx)?)?.add(90)?.iso();
            match generate(&tx, area, through.unwrap_or(&default)) {
                Ok(()) => {
                    tx.commit().map_err(|_| DISK)?;
                    None
                }
                Err(e) => Some(e),
            }
        };
        let mut value = self.organization(area)?;
        value["generationError"] = json!(generation_error);
        let c = self.connection.lock().map_err(|_| DISK)?;
        let rules=c.prepare("SELECT id,title,priority,frequency,start,until_date,active,generated_through,revision FROM recurrence_rules WHERE area=?1 ORDER BY active DESC,id DESC").map_err(|_|DISK)?.query_map([area],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"title":r.get::<_,String>(1)?,"priority":r.get::<_,String>(2)?,"frequency":r.get::<_,String>(3)?,"start":r.get::<_,String>(4)?,"until":r.get::<_,Option<String>>(5)?,"active":r.get::<_,bool>(6)?,"generatedThrough":r.get::<_,Option<String>>(7)?,"revision":r.get::<_,i64>(8)?}))).map_err(|_|DISK)?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|DISK)?;
        value["rules"] = json!(rules);
        let entries=c.prepare("SELECT date,body,revision,updated_at FROM journal WHERE area=?1 ORDER BY date DESC").map_err(|_|DISK)?.query_map([area],|r|Ok(json!({"date":r.get::<_,String>(0)?,"body":r.get::<_,String>(1)?,"revision":r.get::<_,i64>(2)?,"updatedAt":r.get::<_,i64>(3)?}))).map_err(|_|DISK)?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|DISK)?;
        value["journal"] = json!(entries);
        let events=c.prepare("SELECT e.id,e.task,e.title,e.due,e.event,e.at FROM task_events e JOIN tasks t ON t.id=e.task WHERE e.area=?1 AND t.deleted=0 AND e.event IN ('completed','reopened') ORDER BY e.at DESC,e.id DESC").map_err(|_|DISK)?.query_map([area],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"task":r.get::<_,i64>(1)?,"title":r.get::<_,String>(2)?,"due":r.get::<_,String>(3)?,"event":r.get::<_,String>(4)?,"at":r.get::<_,i64>(5)?}))).map_err(|_|DISK)?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|DISK)?;
        value["events"] = json!(events);
        let meetings=c.prepare("SELECT id,title,date,agenda,notes,decisions,actions,revision FROM meetings WHERE area=?1 ORDER BY date DESC,id DESC").map_err(|_|DISK)?.query_map([area],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"title":r.get::<_,String>(1)?,"date":r.get::<_,String>(2)?,"agenda":r.get::<_,String>(3)?,"notes":r.get::<_,String>(4)?,"decisions":r.get::<_,String>(5)?,"actions":serde_json::from_str::<Value>(&r.get::<_,String>(6)?).unwrap_or(Value::Null),"revision":r.get::<_,i64>(7)?}))).map_err(|_|DISK)?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|DISK)?;
        let mut meetings = meetings;
        for m in &mut meetings {
            let converted = c
                .prepare("SELECT action_token FROM meeting_conversions WHERE meeting=?1")
                .map_err(|_| DISK)?
                .query_map([m["id"].as_i64().unwrap()], |r| r.get::<_, String>(0))
                .map_err(|_| DISK)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|_| DISK)?;
            m["converted"] = json!(converted);
        }
        value["meetings"] = json!(meetings);
        Ok(value)
    }
}

pub fn validate_tables(c: &Connection) -> rusqlite::Result<bool> {
    let bad:i64=c.query_row("SELECT (SELECT count(*) FROM tasks t LEFT JOIN recurrence_rules r ON r.id=t.rule_id LEFT JOIN meetings m ON m.id=t.meeting_id WHERE t.scheduled_due IS NULL OR (t.done=1 AND t.cancelled=1) OR (t.completed_at<0 OR t.cancelled_at<0) OR (t.cancelled=0 AND t.cancel_reason IS NOT NULL) OR (t.done=0 AND t.completed_at IS NOT NULL) OR (t.cancelled=0 AND t.cancelled_at IS NOT NULL) OR (t.rule_id IS NOT NULL AND (r.id IS NULL OR r.area!=t.area)) OR (t.meeting_id IS NOT NULL AND (m.id IS NULL OR m.area!=t.area)))+(SELECT count(*) FROM task_events e JOIN tasks t ON t.id=e.task WHERE e.area!=t.area OR e.at<0 OR e.event NOT IN ('completed','reopened','cancelled','resumed'))+(SELECT count(*) FROM journal WHERE revision<0 OR updated_at<0 OR trim(body)='' OR instr(body,char(0))>0 OR length(CAST(body AS BLOB)) NOT BETWEEN 1 AND 65536)+(SELECT count(*) FROM meetings WHERE area!='trabalho' OR revision<0)",[],|r|r.get(0))?;
    let version: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version >= 9 {
        let invalid:i64=c.query_row("SELECT count(*) FROM tasks WHERE cancelled!=0 OR cancelled_at IS NOT NULL OR cancel_reason IS NOT NULL OR (deleted=1 AND deleted_reason IS NULL) OR (deleted=0 AND deleted_reason IS NOT NULL) OR (deleted_reason='rule' AND rule_id IS NULL)",[],|r|r.get(0))?;
        if invalid > 0 {
            return Ok(false);
        }
    }
    if bad > 0 {
        return Ok(false);
    }
    for table in ["recurrence_rules", "meetings"] {
        let mut stmt = c.prepare(&format!("SELECT * FROM {table}"))?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            let area: String = r.get(1)?;
            let title: String = r.get(2)?;
            if organizacao::personal_area(&area).is_err() || organizacao::title(&title).is_err() {
                return Ok(false);
            }
            if table == "recurrence_rules" {
                let a = json!({"title":title,"priority":r.get::<_,String>(3)?,"frequency":r.get::<_,String>(4)?,"due":r.get::<_,String>(5)?,"until":r.get::<_,Option<String>>(6)?});
                if validate_args("criar_recorrencia", &a).is_err()
                    || !organizacao::valid_token(&r.get::<_, String>(9)?)
                {
                    return Ok(false);
                }
                let through: Option<String> = r.get(8)?;
                if through.is_some_and(|v| {
                    organizacao::date(&v).is_err() || v.as_str() < a["due"].as_str().unwrap()
                }) {
                    return Ok(false);
                }
            } else {
                let actions: Value = match serde_json::from_str(&r.get::<_, String>(7)?) {
                    Ok(v) => v,
                    Err(_) => return Ok(false),
                };
                let a = json!({"id":r.get::<_,i64>(0)?,"title":title,"date":r.get::<_,String>(3)?,"agenda":r.get::<_,String>(4)?,"notes":r.get::<_,String>(5)?,"decisions":r.get::<_,String>(6)?,"actions":actions});
                if validate_args("salvar_reuniao", &a).is_err()
                    || !organizacao::valid_token(&r.get::<_, String>(8)?)
                {
                    return Ok(false);
                }
            }
        }
    }
    let conversions=c.prepare("SELECT mc.action_token,m.actions,t.meeting_id,mc.meeting FROM meeting_conversions mc JOIN meetings m ON m.id=mc.meeting LEFT JOIN tasks t ON t.id=mc.task")?.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<i64>>(2)?,r.get::<_,i64>(3)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    for (token, actions, task_meeting, meeting) in conversions {
        let values: Vec<MeetingAction> = match serde_json::from_str(&actions) {
            Ok(v) => v,
            Err(_) => return Ok(false),
        };
        if !organizacao::valid_token(&token)
            || !values.iter().any(|a| a.token == token)
            || task_meeting.is_some_and(|v| v != meeting)
        {
            return Ok(false);
        }
    }
    for query in [
        "SELECT date FROM journal",
        "SELECT scheduled_due FROM tasks",
        "SELECT due FROM task_events",
    ] {
        let values = c
            .prepare(query)?
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if values.iter().any(|s| organizacao::date(s).is_err()) {
            return Ok(false);
        }
    }
    Ok(true)
}
