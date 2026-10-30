use rusqlite::{Connection, OpenFlags};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
const APP_ID: i32 = 0x4c4e5754;
const SCHEMA: &str = include_str!("../migrations/001_work.sql");
const DISK_ERROR: &str =
    "Não foi possível acessar os dados do Lunes Trabalho. Verifique disco e permissões.";
type Result<T> = std::result::Result<T, String>;
pub struct Dados {
    root: PathBuf,
    _lock: fs::File,
    pub(crate) connection: Mutex<Connection>,
}
fn disk<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|_| DISK_ERROR.to_owned())
}
fn theme(value: &str) -> Result<()> {
    match value {
        "claro" | "escuro" => Ok(()),
        _ => Err("Tema inválido. Use claro ou escuro.".into()),
    }
}

pub(crate) fn private_dir(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(path).map_err(|_| DISK_ERROR)?;
    if fs::symlink_metadata(path)
        .map_err(|_| DISK_ERROR)?
        .file_type()
        .is_symlink()
    {
        return Err("O armazenamento do Lunes não pode ser um link simbólico.".into());
    }
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| DISK_ERROR)?;
    Ok(())
}

pub(crate) fn regular(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if !meta.file_type().is_file() => {
            Err("Arquivo fora do escopo de armazenamento do Lunes.".into())
        }
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(DISK_ERROR.into()),
    }
}

impl Dados {
    pub fn open(root: PathBuf) -> Result<Self> {
        if !root.is_absolute() {
            return Err("O diretório de dados deve ser absoluto.".into());
        }
        let repo = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
            .map_err(|_| DISK_ERROR)?;
        let mut ancestor = root.as_path();
        while !ancestor.exists() {
            ancestor = ancestor.parent().ok_or(DISK_ERROR)?;
        }
        if fs::canonicalize(ancestor)
            .map_err(|_| DISK_ERROR)?
            .starts_with(&repo)
        {
            return Err("Dados devem ficar fora do repositório.".into());
        }
        private_dir(&root)?;
        private_dir(&root.join("backups"))?;
        let lock_path = root.join("instance.lock");
        regular(&lock_path)?;
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options.open(&lock_path).map_err(|_| DISK_ERROR)?;
        lock.try_lock().map_err(|_|"Os dados já estão abertos em outra janela do Lunes. Feche a outra instância antes de iniciar.")?;
        let path = root.join("lunes-trabalho.sqlite3");
        regular(&path)?;
        let mut connection = disk(Connection::open(&path))?;
        #[cfg(unix)]
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|_| DISK_ERROR)?;
        disk(connection.busy_timeout(Duration::from_secs(3)))?;
        disk(connection.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL; PRAGMA trusted_schema=OFF;",
        ))?;
        let version: i32 = disk(connection.query_row("PRAGMA user_version", [], |r| r.get(0)))?;
        let existing: i64 = disk(connection.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
            [],
            |r| r.get(0),
        ))?;
        let application: i32 =
            disk(connection.query_row("PRAGMA application_id", [], |r| r.get(0)))?;
        if version == 0 && existing == 0 && application == 0 {
            let tx = disk(connection.transaction())?;
            disk(tx.execute_batch(SCHEMA))?;
            disk(tx.pragma_update(None, "application_id", APP_ID))?;
            disk(tx.pragma_update(None, "user_version", 1))?;
            disk(tx.commit())?;
        }
        validate(&connection)?;
        Ok(Self {
            root,
            _lock: lock,
            connection: Mutex::new(connection),
        })
    }

    pub fn read_theme(&self, legacy: Option<&str>) -> Result<String> {
        if let Some(value) = legacy {
            theme(value)?;
        }
        let mut conn = self.connection.lock().map_err(|_| DISK_ERROR)?;
        let tx = disk(conn.transaction())?;
        disk(tx.execute(
            "INSERT OR IGNORE INTO preferences(key,value) VALUES ('theme',?1)",
            [legacy.unwrap_or("claro")],
        ))?;
        let saved = disk(tx.query_row(
            "SELECT value FROM preferences WHERE key='theme'",
            [],
            |r| r.get(0),
        ))?;
        disk(tx.commit())?;
        Ok(saved)
    }

    pub fn save_theme(&self, value: &str) -> Result<()> {
        theme(value)?;
        let mut conn = self.connection.lock().map_err(|_| DISK_ERROR)?;
        let tx = disk(conn.transaction())?;
        disk(tx.execute("INSERT INTO preferences(key,value) VALUES ('theme',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [value]))?;
        disk(tx.commit())
    }

    pub fn export(&self) -> Result<String> {
        let conn = self.connection.lock().map_err(|_| DISK_ERROR)?;
        validate(&conn)?;
        snapshot(&conn, &self.root)
    }

    pub fn restore(&self, id: &str) -> Result<()> {
        storage_scope(&self.root)?;
        if !id.starts_with("backup-")
            || !id.ends_with(".sqlite3")
            || id.len() > 80
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
        {
            return Err("Identificador de backup inválido.".into());
        }
        let path = self.root.join("backups").join(id);
        regular(&path)?;
        if fs::metadata(&path).map_err(|_| DISK_ERROR)?.len() > 128 * 1024 * 1024 {
            return Err("Backup excede o limite desta versão (128 MiB).".into());
        }
        let imported = disk(Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        ))?;
        disk(imported.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA query_only=ON;"))?;
        validate(&imported)?;
        let tables = [
            ("preferences", 1),
            ("recurrence_rules", 1),
            ("meetings", 1),
            ("tasks", 1),
            ("task_events", 1),
            ("journal", 1),
            ("meeting_conversions", 1),
            ("notes", 1),
            ("goals", 1),
            ("goal_tasks", 1),
        ];
        let version: i32 = disk(imported.query_row("PRAGMA user_version", [], |r| r.get(0)))?;
        let mut records = Vec::new();
        for &(table, since) in &tables {
            if since > version {
                continue;
            }
            let mut stmt = disk(imported.prepare(&format!("SELECT * FROM {table}")))?;
            let columns = stmt.column_count();
            let names = stmt.column_names().join(",");
            let rows = disk(stmt.query_map([], |r| {
                (0..columns)
                    .map(|i| r.get::<_, rusqlite::types::Value>(i))
                    .collect::<rusqlite::Result<Vec<_>>>()
            }))?
            .collect::<rusqlite::Result<Vec<_>>>();
            records.push((table, names, disk(rows)?));
        }
        let mut conn = self.connection.lock().map_err(|_| DISK_ERROR)?;
        snapshot(&conn, &self.root)?;
        let tx = disk(conn.transaction())?;
        for &(table, _) in tables.iter().rev() {
            disk(tx.execute(&format!("DELETE FROM {table}"), []))?;
        }
        for (table, names, rows) in records {
            for row in rows {
                let placeholders = (1..=row.len())
                    .map(|i| format!("?{i}"))
                    .collect::<Vec<_>>()
                    .join(",");
                disk(tx.execute(
                    &format!("INSERT INTO {table} ({names}) VALUES ({placeholders})"),
                    rusqlite::params_from_iter(row),
                ))?;
            }
        }
        validate(&tx)?;
        disk(tx.commit())
    }
}

fn schema(conn: &Connection) -> rusqlite::Result<Vec<(String, String, String)>> {
    conn.prepare("SELECT type,name,sql FROM sqlite_schema WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' ORDER BY type,name")?
        .query_map([], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect()
}
fn validate(conn: &Connection) -> Result<()> {
    let valid = || -> rusqlite::Result<bool> {
        let id: i32 = conn.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        let version: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let integrity: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        let foreign = conn
            .prepare("PRAGMA foreign_key_check")?
            .query([])?
            .next()?
            .is_some();
        let reference = Connection::open_in_memory()?;
        reference.execute_batch(SCHEMA)?;
        Ok(id == APP_ID
            && version == 1
            && integrity == "ok"
            && !foreign
            && schema(conn)? == schema(&reference)?
            && crate::organizacao::validate_records(conn)?
            && crate::agenda::validate_tables(conn)?
            && crate::metas::validate(conn)?)
    };
    if valid().unwrap_or(false) {
        Ok(())
    } else {
        Err("Banco ou backup incompatível com Lunes Trabalho. Nenhum dado foi substituído.".into())
    }
}
fn storage_scope(root: &Path) -> Result<()> {
    for path in [root.to_path_buf(), root.join("backups")] {
        let meta = fs::symlink_metadata(path).map_err(|_| DISK_ERROR)?;
        if !meta.file_type().is_dir() {
            return Err("Diretório fora do escopo do Lunes.".into());
        }
    }
    Ok(())
}

fn snapshot(conn: &Connection, root: &Path) -> Result<String> {
    storage_scope(root)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| DISK_ERROR)?
        .as_nanos();
    let id = format!("backup-{stamp}.sqlite3");
    let path = root.join("backups").join(&id);
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|_| DISK_ERROR)?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|_| DISK_ERROR)?;
    let result = (|| {
        disk(conn.backup(rusqlite::MAIN_DB, &path, None))?;
        file.sync_all().map_err(|_| DISK_ERROR)?;
        #[cfg(unix)]
        fs::File::open(root.join("backups"))
            .and_then(|dir| dir.sync_all())
            .map_err(|_| DISK_ERROR)?;
        Ok(id)
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}
