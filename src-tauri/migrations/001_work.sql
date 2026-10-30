CREATE TABLE preferences (
    key TEXT PRIMARY KEY CHECK (key = 'theme'),
    value TEXT NOT NULL CHECK (value IN ('claro', 'escuro'))
) STRICT;
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    area TEXT NOT NULL CHECK(area='trabalho'),
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 200),
    priority TEXT NOT NULL CHECK(priority IN ('alta','media','baixa')),
    due TEXT NOT NULL CHECK(length(due)=10),
    done INTEGER NOT NULL CHECK(done IN (0,1)),
    token TEXT NOT NULL UNIQUE
);
CREATE TABLE notes (
    id INTEGER PRIMARY KEY,
    area TEXT NOT NULL CHECK(area='trabalho'),
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 200),
    body TEXT NOT NULL CHECK(length(CAST(body AS BLOB)) BETWEEN 1 AND 65536),
    token TEXT NOT NULL UNIQUE
);
ALTER TABLE tasks ADD COLUMN revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0);
ALTER TABLE notes ADD COLUMN revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0);
CREATE TABLE recurrence_rules (
 id INTEGER PRIMARY KEY, area TEXT NOT NULL CHECK(area='trabalho'),
 title TEXT NOT NULL, priority TEXT NOT NULL CHECK(priority IN ('alta','media','baixa')),
 frequency TEXT NOT NULL CHECK(frequency IN ('daily','weekly','monthly')),
 start TEXT NOT NULL, until_date TEXT, active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1)),
 generated_through TEXT, token TEXT NOT NULL UNIQUE, revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0)
);
CREATE TABLE meetings (
 id INTEGER PRIMARY KEY, area TEXT NOT NULL CHECK(area='trabalho'), title TEXT NOT NULL,
 date TEXT NOT NULL, agenda TEXT NOT NULL, notes TEXT NOT NULL, decisions TEXT NOT NULL,
 actions TEXT NOT NULL, token TEXT NOT NULL UNIQUE, revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0)
);
ALTER TABLE tasks ADD COLUMN cancelled INTEGER NOT NULL DEFAULT 0 CHECK(cancelled IN (0,1));
ALTER TABLE tasks ADD COLUMN completed_at INTEGER;
ALTER TABLE tasks ADD COLUMN cancelled_at INTEGER;
ALTER TABLE tasks ADD COLUMN cancel_reason TEXT CHECK(cancel_reason IN ('manual','rule'));
ALTER TABLE tasks ADD COLUMN scheduled_due TEXT;
ALTER TABLE tasks ADD COLUMN rule_id INTEGER REFERENCES recurrence_rules(id);
ALTER TABLE tasks ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0 CHECK(deleted IN (0,1));
ALTER TABLE tasks ADD COLUMN meeting_id INTEGER REFERENCES meetings(id) ON DELETE SET NULL;
UPDATE tasks SET scheduled_due=due;
CREATE UNIQUE INDEX task_occurrence ON tasks(rule_id,scheduled_due) WHERE rule_id IS NOT NULL;
CREATE TABLE task_events (
 id INTEGER PRIMARY KEY, task INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
 area TEXT NOT NULL CHECK(area='trabalho'), title TEXT NOT NULL, due TEXT NOT NULL,
 event TEXT NOT NULL CHECK(event IN ('completed','reopened','cancelled','resumed')),
 at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX task_events_area ON task_events(area,at DESC,id DESC);
CREATE TABLE journal (
 area TEXT NOT NULL CHECK(area='trabalho'), date TEXT NOT NULL,
 body TEXT NOT NULL, revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0),
 updated_at INTEGER NOT NULL DEFAULT (unixepoch()), PRIMARY KEY(area,date)
);
CREATE TABLE meeting_conversions (
 meeting INTEGER NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
 action_token TEXT NOT NULL, task INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
 PRIMARY KEY(meeting,action_token)
);
ALTER TABLE tasks ADD COLUMN deleted_reason TEXT CHECK(deleted_reason IN ('manual','rule'));
UPDATE tasks SET deleted_reason='manual' WHERE deleted=1;
UPDATE tasks SET deleted=1,deleted_reason=CASE WHEN cancel_reason='rule' AND rule_id IS NOT NULL THEN 'rule' ELSE 'manual' END WHERE cancelled=1;
UPDATE tasks SET cancelled=0,cancelled_at=NULL,cancel_reason=NULL;
CREATE TABLE goals (
    id INTEGER PRIMARY KEY,
    area TEXT NOT NULL CHECK(area='trabalho'),
    title TEXT NOT NULL CHECK(length(CAST(trim(title) AS BLOB)) BETWEEN 1 AND 200),
    description TEXT NOT NULL CHECK(length(CAST(description AS BLOB))<=8000),
    due TEXT CHECK(due IS NULL OR length(due)=10),
    token TEXT NOT NULL UNIQUE CHECK(length(token)=36),
    revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0),
    deleted INTEGER NOT NULL DEFAULT 0 CHECK(deleted IN (0,1))
);
CREATE INDEX goals_area ON goals(area,deleted);
CREATE TABLE goal_tasks (
    task_id INTEGER PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
    goal_id INTEGER NOT NULL REFERENCES goals(id) ON DELETE CASCADE
);
CREATE INDEX goal_tasks_goal ON goal_tasks(goal_id);
