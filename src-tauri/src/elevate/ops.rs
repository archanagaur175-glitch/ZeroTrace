//! The elevated-op protocol and its executor.

use crate::core::ZtError;
use serde::{Deserialize, Serialize};
use windows::core::BSTR;

/// One scoped privileged action. Serialized over the named pipe; the helper
/// deserializes and executes; the caller persists the journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "args")]
pub enum ElevOp {
    RegWrite {
        hive: String,
        key_path: String,
        value_name: String,
        value_type: u32,
        data: Vec<u8>,
    },
    RegDelete {
        hive: String,
        key_path: String,
        value_name: String,
    },
    TaskSetEnabled {
        task_name: String,
        enabled: bool,
    },
    FileMove {
        from: String,
        to: String,
    },
    FirewallAdd {
        name: String,
        program: Option<String>,
        host: Option<String>,
    },
    FirewallRemove {
        name: String,
    },
    RestorePoint {
        description: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElevResponse {
    pub ok: bool,
    pub detail: String,
}

impl ElevResponse {
    pub fn err(e: impl std::fmt::Display) -> Self {
        ElevResponse {
            ok: false,
            detail: e.to_string(),
        }
    }
    pub fn ok(detail: impl Into<String>) -> Self {
        ElevResponse {
            ok: true,
            detail: detail.into(),
        }
    }
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn execute(op: &ElevOp) -> ElevResponse {
    let result: crate::core::Result<()> = match op {
        ElevOp::RegWrite {
            hive,
            key_path,
            value_name,
            value_type,
            data,
        } => reg_write(
            hive,
            key_path,
            value_name,
            Some((*value_type, data.clone())),
        ),
        ElevOp::RegDelete {
            hive,
            key_path,
            value_name,
        } => reg_delete(hive, key_path, value_name),
        ElevOp::TaskSetEnabled { task_name, enabled } => task_set_enabled(task_name, *enabled),
        ElevOp::FileMove { from, to } => {
            std::fs::rename(from, to).map_err(|e| ZtError::new(e.to_string()))
        }
        ElevOp::FirewallAdd {
            name,
            program,
            host,
        } => crate::network::firewall::add_block_rule(name, program.as_deref(), host.as_deref()),
        ElevOp::FirewallRemove { name } => crate::network::firewall::remove_rule(name),
        ElevOp::RestorePoint { description } => {
            crate::trust::restore::create(description).map(|_| ())
        }
    };
    match result {
        Ok(()) => ElevResponse::ok(match op {
            ElevOp::RegWrite { value_name, .. } => format!("value '{value_name}' written"),
            ElevOp::RegDelete { value_name, .. } => format!("value '{value_name}' deleted"),
            ElevOp::TaskSetEnabled { enabled, .. } => {
                if *enabled {
                    "task enabled".to_string()
                } else {
                    "task disabled".to_string()
                }
            }
            ElevOp::FileMove { to, .. } => format!("moved to {to}"),
            ElevOp::FirewallAdd { name, .. } => format!("firewall rule '{name}' added"),
            ElevOp::FirewallRemove { name } => format!("firewall rule '{name}' removed"),
            ElevOp::RestorePoint { .. } => "restore point created".to_string(),
        }),
        Err(e) => ElevResponse::err(e),
    }
}

// -- registry via winreg ------------------------------------------------------

use winreg::enums::{
    RegType, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_SET_VALUE,
};
use winreg::RegKey;
use winreg::RegValue;

fn root(hive: &str) -> crate::core::Result<RegKey> {
    match hive {
        "HKCU" => Ok(RegKey::predef(HKEY_CURRENT_USER)),
        "HKLM" => Ok(RegKey::predef(HKEY_LOCAL_MACHINE)),
        "HKCR" => Ok(RegKey::predef(HKEY_CLASSES_ROOT)),
        other => Err(ZtError::new(format!("unknown hive {other}"))),
    }
}

/// value None means "delete"; Some sets raw bytes/type.
pub fn reg_write(
    hive: &str,
    key_path: &str,
    value_name: &str,
    payload: Option<(u32, Vec<u8>)>,
) -> crate::core::Result<()> {
    let Some((value_type, data)) = payload else {
        return reg_delete(hive, key_path, value_name);
    };
    let key = open_for_write(hive, key_path)?;
    key.set_raw_value(
        value_name,
        &RegValue {
            bytes: data,
            vtype: reg_type(value_type),
        },
    )
    .map_err(|e| ZtError::new(e.to_string()))
}

pub fn reg_delete(hive: &str, key_path: &str, value_name: &str) -> crate::core::Result<()> {
    let key = open_for_write(hive, key_path)?;
    match key.delete_value(value_name) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ZtError::new(e.to_string())),
    }
}

fn open_for_write(hive: &str, key_path: &str) -> crate::core::Result<RegKey> {
    let root = root(hive)?;
    root.open_subkey_with_flags(key_path, KEY_SET_VALUE)
        .or_else(|_| root.create_subkey(key_path).map(|(key, _)| key))
        .map_err(|e| ZtError::new(e.to_string()))
}

fn reg_type(v: u32) -> RegType {
    match v {
        1 => RegType::REG_SZ,
        2 => RegType::REG_EXPAND_SZ,
        3 => RegType::REG_BINARY,
        4 => RegType::REG_DWORD,
        7 => RegType::REG_MULTI_SZ,
        11 => RegType::REG_QWORD,
        _ => RegType::REG_BINARY,
    }
}

// -- task scheduler -----------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TaskState {
    pub name: String,
    pub enabled: bool,
}

pub fn list_tasks() -> crate::core::Result<Vec<TaskState>> {
    with_com(|| unsafe {
        let svc: TaskSchedulerService = connect()?;
        let root = svc.GetFolder(&BSTR::from("\\"))?;
        let mut out = Vec::new();
        walk_folder(&root, &mut out);
        Ok(out)
    })
}

pub fn task_set_enabled(name: &str, enabled: bool) -> crate::core::Result<()> {
    with_com(|| unsafe {
        let svc = connect()?;
        let root = svc.GetFolder(&BSTR::from("\\"))?;
        let task = find_task(&root, name)
            .ok_or_else(|| ZtError::new(format!("task '{name}' not found")))?;
        task.SetEnabled(if enabled {
            windows::Win32::Foundation::VARIANT_TRUE
        } else {
            windows::Win32::Foundation::VARIANT_FALSE
        })?;
        Ok(())
    })
}

fn connect() -> crate::core::Result<TaskSchedulerService> {
    unsafe {
        let svc: TaskSchedulerService = windows::Win32::System::Com::CoCreateInstance(
            &TaskScheduler,
            None,
            windows::Win32::System::Com::CLSCTX_INPROC_SERVER,
        )?;
        svc.Connect(
            &variant_empty(),
            &variant_empty(),
            &variant_empty(),
            &variant_empty(),
        )?;
        Ok(svc)
    }
}

fn walk_folder(folder: &ITaskFolder, out: &mut Vec<TaskState>) {
    unsafe {
        if let Ok(tasks) = folder.GetTasks(0) {
            let count = tasks.Count().unwrap_or(0);
            for i in 0..count {
                if let Ok(t) = tasks.get_Item(&variant_i4(i)) {
                    let name = t.Name().map(|b| b.to_string()).unwrap_or_default();
                    let enabled = t
                        .Enabled()
                        .map(|b| b == windows::Win32::Foundation::VARIANT_TRUE)
                        .unwrap_or(false);
                    out.push(TaskState { name, enabled });
                }
            }
        }
        if let Ok(folders) = folder.GetFolders(0) {
            let count = folders.Count().unwrap_or(0);
            for i in 0..count {
                if let Ok(sub) = folders.get_Item(&variant_i4(i)) {
                    walk_folder(&sub, out);
                }
            }
        }
    }
}

fn find_task(folder: &ITaskFolder, name: &str) -> Option<IRegisteredTask> {
    unsafe {
        if let Ok(tasks) = folder.GetTasks(0) {
            let count = tasks.Count().unwrap_or(0);
            for i in 0..count {
                if let Ok(t) = tasks.get_Item(&variant_i4(i)) {
                    if let Ok(n) = t.Name() {
                        if n == name {
                            return Some(t);
                        }
                    }
                }
            }
        }
        if let Ok(folders) = folder.GetFolders(0) {
            let count = folders.Count().unwrap_or(0);
            for i in 0..count {
                if let Ok(sub) = folders.get_Item(&variant_i4(i)) {
                    if let Some(hit) = find_task(&sub, name) {
                        return Some(hit);
                    }
                }
            }
        }
        None
    }
}

fn variant_empty() -> windows::Win32::System::Variant::VARIANT {
    windows::Win32::System::Variant::VARIANT::default()
}

fn variant_i4(v: i32) -> windows::Win32::System::Variant::VARIANT {
    unsafe {
        let mut x = windows::Win32::System::Variant::VARIANT::default();
        (*x.Anonymous.Anonymous).Anonymous.lVal = v;
        (*x.Anonymous.Anonymous).vt = windows::Win32::System::Variant::VT_I4;
        x
    }
}

fn with_com<T>(f: impl FnOnce() -> crate::core::Result<T>) -> crate::core::Result<T> {
    unsafe {
        let hr = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
        let init_ok = hr.0 == 0;
        let result = f();
        if init_ok {
            windows::Win32::System::Com::CoUninitialize();
        }
        result
    }
}

type TaskSchedulerService = windows::Win32::System::TaskScheduler::ITaskService;
type ITaskFolder = windows::Win32::System::TaskScheduler::ITaskFolder;
type IRegisteredTask = windows::Win32::System::TaskScheduler::IRegisteredTask;
use windows::Win32::System::TaskScheduler::TaskScheduler;
