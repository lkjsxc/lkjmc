package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

final class ShopCommandAdapterTest {
    @Test
    void mapsKnownDaemonPurchaseFailureCodes() {
        assertEquals("shop.purchase.insufficient", ShopCommandAdapter.purchaseFailureKey("shop.insufficient_points"));
        assertEquals("shop.purchase.not-found", ShopCommandAdapter.purchaseFailureKey("shop.item_not_found"));
        assertEquals("shop.purchase.unsupported-delivery", ShopCommandAdapter.purchaseFailureKey("shop.unsupported_delivery"));
        assertEquals("shop.purchase.database", ShopCommandAdapter.purchaseFailureKey("database.error"));
        assertEquals("shop.purchase.denied", ShopCommandAdapter.purchaseFailureKey("unexpected"));
    }
}
