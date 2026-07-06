mod helpers;

use helpers::*;
use serde_json::Value;

pub fn instance_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "instances")? {
        let row = as_object(row, "instances[]")?;
        string(row, "id")?;
        string(row, "kind")?;
        string(row, "desiredState")?;
        string(row, "observedState")?;
        bool_field(row, "healthy")?;
        string(row, "connectHost")?;
        integer_or_null(row, "connectPort")?;
        bool_field(row, "proxyRegistrationDesired")?;
        bool_field(row, "proxyRegistered")?;
        bool_field(row, "joinable")?;
        string(row, "joinDisabledReason")?;
        integer(object(row, "presence")?, "playerCount")?;
    }
    Ok(())
}

pub fn home_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "homes")? {
        travel_row(row, "home")?;
    }
    Ok(())
}

pub fn home_get(value: &Value) -> Result<(), String> {
    true_bool(value, "found")?;
    travel_row(value, "home")
}

pub fn warp_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "warps")? {
        travel_row(row, "warp")?;
    }
    Ok(())
}

pub fn shop_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "items")? {
        let row = as_object(row, "items[]")?;
        string(row, "id")?;
        string(row, "titleKey")?;
        string(row, "category")?;
        integer(row, "pricePoints")?;
        string(row, "deliveryKind")?;
        bool_field(row, "deliveryAvailable")?;
        string(row, "disabledReason")?;
        let delivery = object(row, "delivery")?;
        string(delivery, "executor")?;
        string(delivery, "material")?;
        integer(delivery, "amount")?;
    }
    Ok(())
}

pub fn achievement_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "achievements")? {
        let row = as_object(row, "achievements[]")?;
        string(row, "id")?;
        string(row, "titleKey")?;
        string(row, "descriptionKey")?;
        string(row, "categoryPath")?;
        string(row, "iconMaterial")?;
        integer(row, "current")?;
        integer(row, "required")?;
        string(row, "state")?;
        bool_field(row, "hidden")?;
        bool_field(row, "claimable")?;
        bool_field(row, "rewardClaimed")?;
        array_field(row, "rewards")?;
    }
    Ok(())
}

pub fn random_teleport_quote(value: &Value) -> Result<(), String> {
    string(value, "profileId")?;
    string(value, "targetEnvironment")?;
    integer(value, "costPoints")?;
    integer(value, "balance")?;
    integer(value, "cooldownSeconds")?;
    integer(value, "cooldownRemainingSeconds")?;
    integer(value, "minRadius")?;
    integer(value, "maxRadius")?;
    integer(value, "maxAttempts")?;
    bool_field(value, "confirmationRequired")?;
    bool_field(value, "enabled")?;
    bool_field(value, "canAfford")?;
    array_field(value, "allowedWorlds")?;
    array_field(value, "worldCandidates")?;
    Ok(())
}

pub fn settings_get(value: &Value) -> Result<(), String> {
    string(value, "playerUuid")?;
    string(value, "language")?;
    bool_field(value, "hudEnabled")?;
    bool_field(value, "menuEnabled")
}

pub fn points_balance(value: &Value) -> Result<(), String> {
    string(value, "playerUuid")?;
    integer(value, "balance")
}

pub fn kit_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "kits")? {
        let row = as_object(row, "kits[]")?;
        string(row, "id")?;
        string(row, "titleKey")?;
        integer(row, "rewardPoints")?;
        integer(row, "cooldownHours")?;
    }
    Ok(())
}

pub fn vote_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "links")? {
        let row = as_object(row, "links[]")?;
        string(row, "id")?;
        string(row, "titleKey")?;
        string(row, "url")?;
        integer(row, "sortOrder")?;
    }
    Ok(())
}

pub fn mail_inbox(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "messages")? {
        let row = as_object(row, "messages[]")?;
        string(row, "id")?;
        string(row, "senderName")?;
        string(row, "body")?;
        bool_field(row, "read")?;
    }
    Ok(())
}

pub fn report_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "reports")? {
        let row = as_object(row, "reports[]")?;
        string(row, "id")?;
        string(row, "reporterUuid")?;
        string(row, "targetUuid")?;
        string(row, "serverId")?;
        string(row, "reason")?;
        string(row, "status")?;
    }
    Ok(())
}

pub fn daily_status(value: &Value) -> Result<(), String> {
    bool_field(value, "claimedToday")?;
    integer(value, "points")
}

pub fn party_info(value: &Value) -> Result<(), String> {
    true_bool(value, "found")?;
    string(value, "name")?;
    string(value, "role")
}

pub fn adventure_catalog(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "adventures")? {
        let row = as_object(row, "adventures[]")?;
        string(row, "id")?;
        string(row, "titleKey")?;
        string(row, "iconMaterial")?;
        integer(row, "pricePoints")?;
        integer(row, "maxPartySize")?;
        bool_field(row, "enabled")?;
    }
    Ok(())
}

pub fn claim_list(value: &Value) -> Result<(), String> {
    for row in non_empty_array(value, "claims")? {
        let row = as_object(row, "claims[]")?;
        string(row, "id")?;
        string(row, "ownerUuid")?;
        string(row, "ownerName")?;
        string(row, "name")?;
        integer(row, "chunkCount")?;
    }
    Ok(())
}
