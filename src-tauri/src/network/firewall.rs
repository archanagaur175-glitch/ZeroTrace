//! Windows Firewall rules management via the INetFwPolicy2 COM API.
//!
//! Reads are unprivileged; rule mutation runs inside the elevated helper.

use crate::trust::models::FirewallRule;
use windows::core::{Interface, BSTR};
use windows::Win32::Foundation::{VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::NetworkManagement::WindowsFirewall::{
    INetFwPolicy2, INetFwRule, NetFwPolicy2, NetFwRule, NET_FW_ACTION_BLOCK, NET_FW_RULE_DIR_OUT,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Ole::IEnumVARIANT;
use windows::Win32::System::Variant::{VariantClear, VARIANT};

pub const ZT_PREFIX: &str = "ZeroTrace";
pub const PROGRAM_RULE: &str = "ZeroTrace :: Isolate process";
pub const HOST_RULE: &str = "ZeroTrace :: Block host";

pub fn rule_name_for_program(display_name: &str) -> String {
    format!("{PROGRAM_RULE} :: {display_name}")
}

pub fn rule_name_for_host(host: &str) -> String {
    format!("{HOST_RULE} :: {host}")
}

/// Idempotently add an outbound block rule (replaces any existing same-name).
pub fn add_block_rule(
    name: &str,
    program: Option<&str>,
    host: Option<&str>,
) -> crate::core::Result<()> {
    unsafe {
        let policy: INetFwPolicy2 = CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)?;
        let rules = policy.Rules()?;
        let bname = BSTR::from(name);
        let _ = rules.Remove(&bname);
        let rule: INetFwRule = CoCreateInstance(&NetFwRule, None, CLSCTX_INPROC_SERVER)?;
        rule.SetName(&bname)?;
        rule.SetAction(NET_FW_ACTION_BLOCK)?;
        rule.SetDirection(NET_FW_RULE_DIR_OUT)?;
        rule.SetEnabled(VARIANT_TRUE)?;
        if let Some(p) = program {
            rule.SetApplicationName(&BSTR::from(p))?;
            rule.SetRemoteAddresses(&BSTR::from("*"))?;
        }
        if let Some(h) = host {
            rule.SetRemoteAddresses(&BSTR::from(h))?;
        }
        rules.Add(&rule)?;
        Ok(())
    }
}

pub fn remove_rule(name: &str) -> crate::core::Result<()> {
    unsafe {
        let policy: INetFwPolicy2 = CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)?;
        let rules = policy.Rules()?;
        rules.Remove(&BSTR::from(name))?;
        Ok(())
    }
}

pub fn rule_exists(name: &str) -> bool {
    list_rules()
        .map(|rules| rules.iter().any(|r| r.name == name))
        .unwrap_or(false)
}

pub fn list_rules() -> crate::core::Result<Vec<FirewallRule>> {
    unsafe {
        let policy: INetFwPolicy2 = CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)?;
        let rules = policy.Rules()?;
        let unknown = rules._NewEnum()?;
        let iter: IEnumVARIANT = unknown.cast()?;
        let mut out = Vec::new();
        loop {
            let mut slot: [VARIANT; 1] = [VARIANT::default()];
            let mut fetched: u32 = 0;
            let rc = iter.Next(&mut slot, &mut fetched);
            if rc.is_err() || fetched == 0 {
                break;
            }
            let v = &mut slot[0];
            let punk = (*v.Anonymous.Anonymous.Anonymous.punkVal).clone();
            if let Some(unk) = punk {
                if let Ok(rule) = unk.cast::<INetFwRule>() {
                    out.push(read_rule(&rule));
                }
            }
            let _ = VariantClear(v);
        }
        Ok(out)
    }
}

fn read_rule(rule: &INetFwRule) -> FirewallRule {
    unsafe {
        let name = rule.Name().map(|b| b.to_string()).unwrap_or_default();
        let program = rule
            .ApplicationName()
            .map(|b| b.to_string())
            .ok()
            .filter(|s| !s.is_empty() && s != "*");
        let remote = rule
            .RemoteAddresses()
            .map(|b| b.to_string())
            .ok()
            .filter(|s| !s.is_empty());
        let direction = rule.Direction().map(|d| d.0).unwrap_or(0);
        let action = rule.Action().map(|a| a.0).unwrap_or(1);
        let enabled = rule.Enabled().map(|b| b != VARIANT_FALSE).unwrap_or(false);
        FirewallRule {
            name,
            program,
            remote_addresses: remote,
            direction: if direction == NET_FW_RULE_DIR_OUT.0 {
                "out".into()
            } else {
                "in".into()
            },
            action: if action == NET_FW_ACTION_BLOCK.0 {
                "block".into()
            } else {
                "allow".into()
            },
            enabled,
        }
    }
}
