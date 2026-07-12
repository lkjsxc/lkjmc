//! Isolated E-CONTRACT Rust-owned descriptor candidate; not product code.

#[derive(Clone, Copy)]
struct Field {
    name: &'static str,
    kind: &'static str,
}

trait Request {
    const NAME: &'static str;
    const FIELDS: &'static [Field];
}

struct Status;
impl Request for Status {
    const NAME: &'static str = "status";
    const FIELDS: &'static [Field] = &[];
}

struct InstanceStart {
    id: String,
}
impl Request for InstanceStart {
    const NAME: &'static str = "instance.start";
    const FIELDS: &'static [Field] = &[Field { name: "id", kind: "string" }];
}

struct PlayerTransferSaved {
    player_uuid: String,
}
impl Request for PlayerTransferSaved {
    const NAME: &'static str = "player.transfer.saved";
    const FIELDS: &'static [Field] = &[Field { name: "playerUuid", kind: "uuid" }];
}

struct PlayerShopPurchase {
    player_uuid: String,
    name: String,
    item_id: String,
    correlation_id: String,
}
impl Request for PlayerShopPurchase {
    const NAME: &'static str = "player.shop.purchase";
    const FIELDS: &'static [Field] = &[
        Field { name: "playerUuid", kind: "uuid" },
        Field { name: "name", kind: "string" },
        Field { name: "itemId", kind: "string" },
        Field { name: "correlationId", kind: "uuid" },
    ];
}

fn row<T: Request>() -> String {
    let fields = T::FIELDS.iter().map(|field| format!("[\"{}\",\"{}\"]", field.name, field.kind))
        .collect::<Vec<_>>().join(",");
    format!("[\"{}\",[{}]]", T::NAME, fields)
}

fn main() {
    println!("[{}]", [row::<Status>(), row::<InstanceStart>(), row::<PlayerTransferSaved>(), row::<PlayerShopPurchase>()].join(","));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_follow_the_typed_requests() {
        assert!(row::<InstanceStart>().contains("id"));
        assert!(row::<PlayerShopPurchase>().contains("correlationId"));
    }
}
