use crate::error::XdrError;
use stellar_xdr::{
    ConfigSettingEntry, ConfigSettingId, FreezeBypassTxs, FreezeBypassTxsDelta, FrozenLedgerKeys,
    FrozenLedgerKeysDelta, LedgerKey, LedgerKeyConfigSetting,
};

pub fn frozen_ledger_keys_key() -> LedgerKey {
    LedgerKey::ConfigSetting(LedgerKeyConfigSetting {
        config_setting_id: ConfigSettingId::FrozenLedgerKeys,
    })
}

pub fn frozen_ledger_keys_delta_key() -> LedgerKey {
    LedgerKey::ConfigSetting(LedgerKeyConfigSetting {
        config_setting_id: ConfigSettingId::FrozenLedgerKeysDelta,
    })
}

pub fn freeze_bypass_txs_key() -> LedgerKey {
    LedgerKey::ConfigSetting(LedgerKeyConfigSetting {
        config_setting_id: ConfigSettingId::FreezeBypassTxs,
    })
}

pub fn freeze_bypass_txs_delta_key() -> LedgerKey {
    LedgerKey::ConfigSetting(LedgerKeyConfigSetting {
        config_setting_id: ConfigSettingId::FreezeBypassTxsDelta,
    })
}

pub fn decode_frozen_ledger_keys(
    entry: &ConfigSettingEntry,
) -> Result<&FrozenLedgerKeys, XdrError> {
    if let ConfigSettingEntry::FrozenLedgerKeys(keys) = entry {
        Ok(keys)
    } else {
        Err(XdrError::MalformedInput)
    }
}

pub fn decode_frozen_ledger_keys_delta(
    entry: &ConfigSettingEntry,
) -> Result<&FrozenLedgerKeysDelta, XdrError> {
    if let ConfigSettingEntry::FrozenLedgerKeysDelta(delta) = entry {
        Ok(delta)
    } else {
        Err(XdrError::MalformedInput)
    }
}

pub fn decode_freeze_bypass_txs(entry: &ConfigSettingEntry) -> Result<&FreezeBypassTxs, XdrError> {
    if let ConfigSettingEntry::FreezeBypassTxs(txs) = entry {
        Ok(txs)
    } else {
        Err(XdrError::MalformedInput)
    }
}

pub fn decode_freeze_bypass_txs_delta(
    entry: &ConfigSettingEntry,
) -> Result<&FreezeBypassTxsDelta, XdrError> {
    if let ConfigSettingEntry::FreezeBypassTxsDelta(delta) = entry {
        Ok(delta)
    } else {
        Err(XdrError::MalformedInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_setting_keys() {
        let key1 = frozen_ledger_keys_key();
        assert!(matches!(
            key1,
            LedgerKey::ConfigSetting(LedgerKeyConfigSetting {
                config_setting_id: ConfigSettingId::FrozenLedgerKeys
            })
        ));
    }
}
