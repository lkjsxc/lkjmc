package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

final class ShopCommandAdapterTest {
    @Test
    void mapsKnownDaemonPurchaseFailureCodes() {
        assertEquals("shop.purchase.insufficient", ShopCommandAdapter.purchaseFailureKey("shop.insufficient_points"));
        assertEquals("shop.purchase.not-found", ShopCommandAdapter.purchaseFailureKey("shop.item_not_found"));
        assertEquals("shop.purchase.unsupported-delivery", ShopCommandAdapter.purchaseFailureKey("shop.unsupported_delivery"));
        assertEquals("shop.purchase.database", ShopCommandAdapter.purchaseFailureKey("database.error"));
        assertEquals("shop.purchase.failed", ShopCommandAdapter.purchaseFailureKey("unexpected"));
    }

    @Test
    void delivered_replay_is_neither_delivered_nor_refunded() {
        var replay = JsonParser.parseString("""
            {"duplicate":true,"refundable":false,
             "delivery":{"executor":"minecraft-item","material":"STONE","amount":1}}
            """).getAsJsonObject();

        assertEquals(ShopCommandAdapter.PurchaseAction.REPLAY, ShopCommandAdapter.purchaseAction(replay));
    }

    @Test
    void missing_delivery_without_refund_eligibility_is_contained() {
        var response = JsonParser.parseString("{\"duplicate\":false,\"refundable\":false}").getAsJsonObject();

        assertEquals(ShopCommandAdapter.PurchaseAction.CONTAINED,
            ShopCommandAdapter.purchaseAction(response));
    }

    @Test
    void transfer_intent_reports_pending_not_purchase_completion() {
        assertEquals("shop.purchase.transfer-pending", ShopCommandAdapter.transferOutcome(true));
        assertEquals("shop.purchase.delivery-contained", ShopCommandAdapter.transferOutcome(false));
        assertNotEquals("shop.purchase.ok", ShopCommandAdapter.transferOutcome(true));
    }
}
