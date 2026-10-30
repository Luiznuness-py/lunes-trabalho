use crate::{dados::Dados, organizacao};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};

type Result<T> = std::result::Result<T, String>;
const DISK: &str = "Não foi possível acessar suas metas. Verifique disco e permissões.";

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Goal {
    pub id: Option<i64>,
    pub title: String,
    pub description: String,
    pub due: Option<String>,
    pub token: String,
}
fn valid(goal: &Goal) -> Result<()> {
    organizacao::title(&goal.title)?;
    if goal.id.is_some_and(|id| id <= 0)
        || goal.description.len() > 8000
        || goal.description.contains('\0')
        || !organizacao::valid_token(&goal.token)
    {
        return Err("Meta inválida. Use uma descrição de até 8.000 bytes.".into());
    }
    if let Some(due) = &goal.due {
        organizacao::date(due)?;
    }
    Ok(())
}
fn revision(c: &Connection, area: &str, id: i64, expected: i64) -> Result<()> {
    let actual: i64 = c
        .query_row(
            "SELECT revision FROM goals WHERE id=?1 AND area=?2 AND deleted=0",
            params![id, area],
            |r| r.get(0),
        )
        .map_err(|_| "Meta ausente nesta área.")?;
    if actual != expected {
        return Err("A meta mudou. Atualize antes de salvar.".into());
    }
    Ok(())
}
impl Dados {
    pub fn goals(&self, area: &str, goal_id: Option<i64>) -> Result<Value> {
        organizacao::personal_area(area)?;
        if goal_id.is_some_and(|id| id <= 0) {
            return Err("Meta inválida.".into());
        }
        let c = self.connection.lock().map_err(|_| DISK)?;
        let goals=c.prepare("SELECT g.id,g.title,g.description,g.due,g.token,g.revision,count(t.id),coalesce(sum(t.done),0) FROM goals g LEFT JOIN goal_tasks l ON l.goal_id=g.id LEFT JOIN tasks t ON t.id=l.task_id AND t.area=g.area AND t.deleted=0 WHERE g.area=?1 AND g.deleted=0 GROUP BY g.id ORDER BY (count(t.id)>0 AND count(t.id)=sum(t.done)),g.due IS NULL,g.due,g.id DESC").map_err(|_|DISK)?.query_map([area],|r| {
            let total:i64=r.get(6)?; let completed:i64=r.get(7)?;
            Ok(json!({"id":r.get::<_,i64>(0)?,"title":r.get::<_,String>(1)?,"description":r.get::<_,String>(2)?,"due":r.get::<_,Option<String>>(3)?,"token":r.get::<_,String>(4)?,"revision":r.get::<_,i64>(5)?,"total":total,"completed":completed,"progress":if total==0 {0}else{completed*100/total}}))
        }).map_err(|_|DISK)?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|DISK)?;
        let selected = goal_id
            .filter(|id| goals.iter().any(|g| g["id"].as_i64() == Some(*id)))
            .or_else(|| goals.first().and_then(|g| g["id"].as_i64()));
        let tasks=c.prepare("SELECT t.id,t.title,t.priority,t.due,t.done,t.token,t.cancelled,t.completed_at,t.cancelled_at,t.scheduled_due,t.rule_id,t.meeting_id,t.revision FROM tasks t JOIN goal_tasks l ON l.task_id=t.id JOIN goals g ON g.id=l.goal_id WHERE g.id=?1 AND g.area=?2 AND g.deleted=0 AND t.area=?2 AND t.deleted=0 ORDER BY t.done,t.due,CASE t.priority WHEN 'alta' THEN 0 WHEN 'media' THEN 1 ELSE 2 END,t.id").map_err(|_|DISK)?.query_map(params![selected,area],organizacao::task_view).map_err(|_|DISK)?.collect::<rusqlite::Result<Vec<_>>>().map_err(|_|DISK)?;
        Ok(json!({"area":area,"goals":goals,"selected":selected,"tasks":tasks}))
    }
    pub fn save_goal(&self, area: &str, goal: &Goal, expected: Option<i64>) -> Result<i64> {
        organizacao::personal_area(area)?;
        valid(goal)?;
        let mut c = self.connection.lock().map_err(|_| DISK)?;
        let tx = c.transaction().map_err(|_| DISK)?;
        let id = if let Some(id) = goal.id {
            revision(
                &tx,
                area,
                id,
                expected.ok_or("Reabra a meta antes de editar.")?,
            )?;
            let changed=tx.execute("UPDATE goals SET title=?3,description=?4,due=?5,revision=revision+1 WHERE id=?1 AND area=?2 AND token=?6",params![id,area,goal.title.trim(),goal.description.trim(),goal.due,goal.token]).map_err(|_|DISK)?;
            if changed != 1 {
                return Err("Identificador da meta desatualizado.".into());
            }
            id
        } else {
            tx.execute("INSERT OR IGNORE INTO goals(area,title,description,due,token) VALUES(?1,?2,?3,?4,?5)",params![area,goal.title.trim(),goal.description.trim(),goal.due,goal.token]).map_err(|_|DISK)?;
            tx.query_row("SELECT id FROM goals WHERE token=?1 AND area=?2 AND deleted=0 AND title=?3 AND description=?4 AND due IS ?5",params![goal.token,area,goal.title.trim(),goal.description.trim(),goal.due],|r|r.get(0)).map_err(|_|"Tentativa já utilizada em outra meta. Atualize e tente novamente.")?
        };
        tx.commit().map_err(|_| DISK)?;
        Ok(id)
    }
    pub fn delete_goal(&self, area: &str, id: i64, expected: i64) -> Result<()> {
        organizacao::personal_area(area)?;
        let mut c = self.connection.lock().map_err(|_| DISK)?;
        let tx = c.transaction().map_err(|_| DISK)?;
        revision(&tx, area, id, expected)?;
        tx.execute("DELETE FROM goal_tasks WHERE goal_id=?1", [id])
            .map_err(|_| DISK)?;
        tx.execute(
            "UPDATE goals SET deleted=1,revision=revision+1 WHERE id=?1",
            [id],
        )
        .map_err(|_| DISK)?;
        tx.commit().map_err(|_| DISK.to_string())
    }
    pub fn save_goal_task(
        &self,
        area: &str,
        goal_id: i64,
        task: &organizacao::Task,
        expected: i64,
    ) -> Result<i64> {
        organizacao::personal_area(area)?;
        organizacao::task_valid(task)?;
        if task.id.is_some() {
            return Err("Use uma nova tarefa para esta meta.".into());
        }
        let mut c = self.connection.lock().map_err(|_| DISK)?;
        let tx = c.transaction().map_err(|_| DISK)?;
        revision(&tx, area, goal_id, expected)?;
        let prior: Option<i64> = tx
            .query_row("SELECT id FROM tasks WHERE token=?1", [&task.token], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|_| DISK)?;
        if let Some(id) = prior {
            let same:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks t JOIN goal_tasks l ON l.task_id=t.id WHERE t.id=?1 AND t.area=?2 AND l.goal_id=?3 AND t.deleted=0 AND t.title=?4 AND t.due=?5 AND t.priority=?6 AND t.done=?7)",params![id,area,goal_id,task.title.trim(),task.due,task.priority,task.done],|r|r.get(0)).map_err(|_|DISK)?;
            return if same {
                Ok(id)
            } else {
                Err("Essa tentativa já foi utilizada. Atualize antes de criar outra tarefa.".into())
            };
        }
        let id = organizacao::write_task(&tx, area, task)?;
        tx.execute(
            "INSERT INTO goal_tasks(task_id,goal_id) VALUES(?1,?2)",
            params![id, goal_id],
        )
        .map_err(|_| DISK)?;
        tx.commit().map_err(|_| DISK)?;
        Ok(id)
    }
}
pub fn validate(c: &Connection) -> rusqlite::Result<bool> {
    let mut stmt = c.prepare("SELECT id,title,description,due,token FROM goals")?;
    let goals = stmt.query_map([], |r| {
        Ok(Goal {
            id: r.get(0)?,
            title: r.get(1)?,
            description: r.get(2)?,
            due: r.get(3)?,
            token: r.get(4)?,
        })
    })?;
    for goal in goals {
        if valid(&goal?).is_err() {
            return Ok(false);
        }
    }
    let bad:i64=c.query_row("SELECT count(*) FROM goal_tasks l JOIN goals g ON g.id=l.goal_id JOIN tasks t ON t.id=l.task_id WHERE g.area!=t.area OR g.deleted=1",[],|r|r.get(0))?;
    let invalid: i64 = c.query_row(
        "SELECT count(*) FROM goals WHERE area != 'trabalho' OR revision<1 OR deleted NOT IN (0,1)",
        [],
        |r| r.get(0),
    )?;
    Ok(bad == 0 && invalid == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn goals_reject_invalid_fields_and_dates() {
        let mut goal = Goal {
            id: None,
            title: "Uma meta".into(),
            description: String::new(),
            due: None,
            token: "00000000-0000-4000-8000-000000000180".into(),
        };
        assert!(valid(&goal).is_ok());
        goal.due = Some("2026-02-30".into());
        assert!(valid(&goal).is_err());
        goal.due = None;
        goal.title = " ".into();
        assert!(valid(&goal).is_err());
        goal.title = "Uma meta".into();
        goal.description = "a".repeat(8001);
        assert!(valid(&goal).is_err());
        goal.description = String::new();
        goal.token = "inválido".into();
        assert!(valid(&goal).is_err());
        assert!(serde_json::from_value::<Goal>(json!({"id":null,"title":"Meta","description":"","due":null,"token":"00000000-0000-4000-8000-000000000180","approved":true})).is_err());
    }
}
